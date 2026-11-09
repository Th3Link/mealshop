//! Immutable, bounded household raster assets. No remote image fetching or active formats.
use crate::household::{self, Shared};
use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path as FsPath,
};
const MAX_BYTES: usize = 5 * 1024 * 1024;
pub(crate) fn valid_id(id: &str) -> bool {
    id.strip_prefix("image-")
        .and_then(|rest| rest.split_once('.'))
        .is_some_and(|(number, ext)| {
            number.len() == 8
                && number.bytes().all(|b| b.is_ascii_digit())
                && matches!(ext, "jpg" | "png" | "webp")
        })
}
fn dimensions(data: &[u8], mime: &str) -> Option<(u32, u32, &'static str)> {
    match mime {
        "image/png"
            if data.len() >= 33
                && &data[..8] == b"\x89PNG\r\n\x1a\n"
                && &data[12..16] == b"IHDR" =>
        {
            Some((
                u32::from_be_bytes(data[16..20].try_into().ok()?),
                u32::from_be_bytes(data[20..24].try_into().ok()?),
                "png",
            ))
        }
        "image/jpeg" if data.starts_with(b"\xff\xd8") && data.ends_with(b"\xff\xd9") => {
            let mut offset = 2;
            while offset + 4 < data.len() {
                if data[offset] != 0xff {
                    return None;
                }
                while data.get(offset) == Some(&0xff) {
                    offset += 1;
                }
                let marker = *data.get(offset)?;
                offset += 1;
                if marker == 0xda || marker == 0xd9 {
                    return None;
                }
                if marker == 1 || (0xd0..=0xd8).contains(&marker) {
                    continue;
                }
                let length =
                    u16::from_be_bytes(data.get(offset..offset + 2)?.try_into().ok()?) as usize;
                if length < 2 || offset + length > data.len() {
                    return None;
                }
                if matches!(marker,0xc0..=0xc3|0xc5..=0xc7|0xc9..=0xcb|0xcd..=0xcf) && length >= 7 {
                    let height = u16::from_be_bytes(data[offset + 3..offset + 5].try_into().ok()?);
                    let width = u16::from_be_bytes(data[offset + 5..offset + 7].try_into().ok()?);
                    return Some((u32::from(width), u32::from(height), "jpg"));
                }
                offset += length;
            }
            None
        }
        "image/webp"
            if data.len() >= 30
                && &data[..4] == b"RIFF"
                && &data[8..12] == b"WEBP"
                && u32::from_le_bytes(data[4..8].try_into().ok()?) as usize + 8 == data.len() =>
        {
            let (width, height) = match &data[12..16] {
                b"VP8X" if data[20] & 2 == 0 => (
                    1 + u32::from_le_bytes([data[24], data[25], data[26], 0]),
                    1 + u32::from_le_bytes([data[27], data[28], data[29], 0]),
                ),
                b"VP8L" if data[20] == 0x2f => {
                    let bits = u32::from_le_bytes(data[21..25].try_into().ok()?);
                    (1 + (bits & 0x3fff), 1 + ((bits >> 14) & 0x3fff))
                }
                b"VP8 " if &data[23..26] == b"\x9d\x01\x2a" => (
                    u32::from(u16::from_le_bytes(data[26..28].try_into().ok()?) & 0x3fff),
                    u32::from(u16::from_le_bytes(data[28..30].try_into().ok()?) & 0x3fff),
                ),
                _ => return None,
            };
            Some((width, height, "webp"))
        }
        _ => None,
    }
}
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route(
            "/api/household/images",
            axum::routing::post(upload).layer(DefaultBodyLimit::max(MAX_BYTES)),
        )
        .route("/api/household/images/{id}", get(read))
}
fn error(status: StatusCode, code: &str) -> Response {
    household::reply(status, json!({"code":code}))
}
async fn upload(State(state): State<Shared>, headers: HeaderMap, data: Bytes) -> Response {
    if !household::can_write(&state, &headers) {
        return error(StatusCode::FORBIDDEN, "household_write_required");
    }
    let mime = headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let Some((width, height, ext)) = dimensions(&data, mime) else {
        return error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_image");
    };
    if data.len() > MAX_BYTES
        || width == 0
        || height == 0
        || width > 8192
        || height > 8192
        || u64::from(width) * u64::from(height) > 32_000_000
    {
        return error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_image");
    }
    tokio::task::spawn_blocking(move || {
        let Ok(_guard) = state.store.lock() else {
            return error(StatusCode::SERVICE_UNAVAILABLE, "image_storage_failed");
        };
        match persist(&state.images, &data, ext) {
            Ok(id) => household::reply(StatusCode::CREATED, json!({"id":id})),
            Err(code) => error(StatusCode::SERVICE_UNAVAILABLE, code),
        }
    })
    .await
    .unwrap_or_else(|_| error(StatusCode::SERVICE_UNAVAILABLE, "image_storage_failed"))
}
fn persist(directory: &FsPath, data: &[u8], ext: &str) -> Result<String, &'static str> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(directory)
        .map_err(|_| "image_storage_failed")?;
    let entries = fs::read_dir(directory).map_err(|_| "image_storage_failed")?;
    let mut next = 1;
    let mut size = 0;
    for entry in entries {
        let entry = entry.map_err(|_| "image_storage_failed")?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("image_storage_failed")?;
        if !valid_id(name)
            || !entry
                .file_type()
                .map_err(|_| "image_storage_failed")?
                .is_file()
        {
            return Err("image_storage_failed");
        }
        next = next.max(
            name[6..14]
                .parse::<u32>()
                .map_err(|_| "image_storage_failed")?
                + 1,
        );
        size += entry.metadata().map_err(|_| "image_storage_failed")?.len();
    }
    if next > 1000 || size + data.len() as u64 > 100 * 1024 * 1024 {
        return Err("image_capacity");
    }
    let pending = directory.join(".image.pending");
    let id = format!("image-{next:08}.{ext}");
    let write = || -> std::io::Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&pending)?;
        file.write_all(data)?;
        file.sync_all()?;
        fs::hard_link(&pending, directory.join(&id))?;
        fs::remove_file(pending)?;
        #[cfg(unix)]
        File::open(directory)?.sync_all()?;
        Ok(())
    };
    write().map_err(|_| "image_storage_failed")?;
    Ok(id)
}
async fn read(State(state): State<Shared>, Path(id): Path<String>, headers: HeaderMap) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    if !valid_id(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    tokio::task::spawn_blocking(move || {
        let Ok(file) = File::open(state.images.join(&id)) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let mut data = vec![];
        if file
            .take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut data)
            .is_err()
            || data.len() > MAX_BYTES
        {
            return error(StatusCode::SERVICE_UNAVAILABLE, "image_storage_failed");
        }
        let mime = match FsPath::new(&id)
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("png") => "image/png",
            Some("jpg") => "image/jpeg",
            _ => "image/webp",
        };
        (
            [
                (header::CONTENT_TYPE, mime),
                (header::CACHE_CONTROL, "no-store"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                (header::CONTENT_DISPOSITION, "inline"),
            ],
            data,
        )
            .into_response()
    })
    .await
    .unwrap_or_else(|_| error(StatusCode::SERVICE_UNAVAILABLE, "image_storage_failed"))
}
