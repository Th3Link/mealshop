//! Unreviewed recipe candidates handed in by assistants and customer-operated companions.
//! A suggestion is source text waiting for a cook; it is never a recipe and never calculated.
use crate::{
    household::{self, Authority, Shared},
    imports::Candidate,
    recipe_origin::Origin,
};
use axum::{
    Extension, Router,
    extract::{Path as Segment, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write as _;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

const CAPACITY: usize = 50;
const MAX_BYTES: usize = 65_536;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Suggestion {
    pub(crate) id: String,
    pub(crate) actor: String,
    pub(crate) received_at: i64,
    pub(crate) source_url: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) original: Candidate,
}
impl Suggestion {
    fn valid(&self) -> bool {
        let id = self.id.strip_prefix("suggestion-");
        id.is_some_and(|hex| hex.len() == 32 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
            && crate::membership::valid_user(&self.actor)
            && self.received_at >= 0
            && content_valid(
                &self.original,
                self.source_url.as_deref(),
                self.note.as_deref(),
            )
    }
}
fn content_valid(original: &Candidate, source_url: Option<&str>, note: Option<&str>) -> bool {
    original.valid()
        && (original
            .title
            .as_ref()
            .is_some_and(|t| !t.trim().is_empty())
            || !original.ingredients.is_empty()
            || !original.steps.is_empty())
        && source_url.is_none_or(|url| {
            Origin::Imported {
                source_url: url.to_owned(),
            }
            .validate()
            .is_ok()
        })
        && note.is_none_or(|n| {
            n.len() <= 4000
                && !n
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        })
}
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Data {
    schema_version: u32,
    revision: u64,
    suggestions: Vec<Suggestion>,
}
pub(crate) struct Store {
    path: PathBuf,
    data: Data,
    writable: bool,
}
impl Store {
    pub(crate) fn open(directory: &Path) -> Result<Self, String> {
        // The inbox is replaced by rename; a leftover pending file was never published.
        let pending = directory.join(".suggestions.pending");
        if pending.exists() {
            fs::remove_file(&pending).map_err(|_| "Cannot clear interrupted suggestion write")?;
        }
        let path = directory.join("suggestions.json");
        let data = if path.exists() {
            let mut bytes = Vec::new();
            File::open(&path)
                .and_then(|f| f.take(4_194_305).read_to_end(&mut bytes))
                .map_err(|_| "Cannot read recipe suggestions")?;
            if bytes.len() > 4_194_304 {
                return Err("Recipe suggestions exceed 4 MiB".to_owned());
            }
            serde_json::from_slice(&bytes).map_err(|_| "Invalid recipe suggestions")?
        } else {
            Data {
                schema_version: 1,
                ..Data::default()
            }
        };
        if data.schema_version != 1
            || data.suggestions.len() > CAPACITY
            || !data.suggestions.iter().all(Suggestion::valid)
        {
            return Err("Invalid recipe suggestions".to_owned());
        }
        Ok(Self {
            path,
            data,
            writable: true,
        })
    }
    fn persist(&mut self, data: Data) -> Result<(), &'static str> {
        if !self.writable {
            return Err("suggestion_storage_failed");
        }
        let bytes = serde_json::to_vec(&data).map_err(|_| "suggestion_storage_failed")?;
        let pending = self.path.with_file_name(".suggestions.pending");
        let write = || -> std::io::Result<()> {
            let mut options = OpenOptions::new();
            options.create_new(true).write(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&pending)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&pending, &self.path)?;
            File::open(
                self.path
                    .parent()
                    .ok_or_else(|| std::io::Error::other("parent"))?,
            )?
            .sync_all()
        };
        if write().is_err() {
            self.writable = false;
            return Err("suggestion_storage_failed");
        }
        self.data = data;
        Ok(())
    }
    pub(crate) fn list(&self) -> &[Suggestion] {
        &self.data.suggestions
    }
    pub(crate) fn find(&self, id: &str) -> Option<&Suggestion> {
        self.data.suggestions.iter().find(|s| s.id == id)
    }
    /// Stores a candidate and reports whether the same content was already waiting.
    pub(crate) fn add(
        &mut self,
        actor: &str,
        received_at: i64,
        source_url: Option<String>,
        note: Option<String>,
        original: Candidate,
    ) -> Result<(String, bool), &'static str> {
        if !content_valid(&original, source_url.as_deref(), note.as_deref()) {
            return Err("suggestion_invalid");
        }
        if let Some(existing) = self.data.suggestions.iter().find(|s| {
            s.actor == actor
                && s.source_url == source_url
                && s.note == note
                && s.original == original
        }) {
            return Ok((existing.id.clone(), true));
        }
        if self.data.suggestions.len() >= CAPACITY {
            return Err("suggestion_capacity");
        }
        let mut bytes = [0u8; 16];
        File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut bytes))
            .map_err(|_| "suggestion_storage_failed")?;
        let mut id = String::from("suggestion-");
        for b in bytes {
            write!(&mut id, "{b:02x}").map_err(|_| "suggestion_storage_failed")?;
        }
        let suggestion = Suggestion {
            id: id.clone(),
            actor: actor.to_owned(),
            received_at,
            source_url,
            note,
            original,
        };
        if !suggestion.valid()
            || serde_json::to_vec(&suggestion).map_or(true, |v| v.len() > MAX_BYTES)
        {
            return Err("suggestion_invalid");
        }
        let mut data = self.data.clone();
        data.revision = data.revision.checked_add(1).ok_or("suggestion_capacity")?;
        data.suggestions.push(suggestion);
        self.persist(data)?;
        Ok((id, false))
    }
    pub(crate) fn remove(&mut self, id: &str) -> Result<(), &'static str> {
        let mut data = self.data.clone();
        let before = data.suggestions.len();
        data.suggestions.retain(|s| s.id != id);
        if data.suggestions.len() == before {
            return Err("suggestion_not_found");
        }
        data.revision = data.revision.checked_add(1).ok_or("suggestion_capacity")?;
        self.persist(data)
    }
}

pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/suggestions", get(list))
        .route("/api/household/suggestions/{id}", delete(discard))
}
fn failure(code: &'static str) -> Response {
    household::reply(
        match code {
            "suggestion_write_required" => StatusCode::FORBIDDEN,
            "suggestion_not_found" => StatusCode::NOT_FOUND,
            "suggestion_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        },
        json!({"code":code}),
    )
}
/// The refusal for anyone who may not read or change the inbox.
fn refused(
    state: &Shared,
    headers: &HeaderMap,
    who: Option<Extension<Authority>>,
) -> Option<Response> {
    if !household::authorized(state, headers) {
        return Some(household::denied());
    }
    let actor = household::authority(state, headers, who);
    if household::can_write(state, headers) && actor.role != crate::membership::Role::Reader {
        None
    } else {
        Some(failure("suggestion_write_required"))
    }
}
async fn list(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
) -> Response {
    if let Some(response) = refused(&state, &headers, who) {
        return response;
    }
    match state.suggestions.lock() {
        Ok(store) => household::reply(
            StatusCode::OK,
            json!({"revision":store.data.revision,"capacity":CAPACITY,"suggestions":store.list()}),
        ),
        Err(_) => failure("suggestion_storage_failed"),
    }
}
async fn discard(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
    Segment(id): Segment<String>,
) -> Response {
    if let Some(response) = refused(&state, &headers, who) {
        return response;
    }
    let result = state
        .suggestions
        .lock()
        .map_err(|_| "suggestion_storage_failed")
        .and_then(|mut store| store.remove(&id));
    match result {
        Ok(()) => household::reply(StatusCode::OK, json!({"discarded":id})),
        Err(code) => failure(code),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    fn candidate(title: &str) -> Candidate {
        Candidate {
            title: Some(title.to_owned()),
            ingredients: vec!["2 tbsp olive oil".to_owned()],
            ..Candidate::default()
        }
    }
    fn directory(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "mealshop-suggestions-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn suggestions_persist_deduplicate_and_leave_only_on_discard() {
        let dir = directory("persist");
        let mut store = Store::open(&dir).unwrap();
        let url = Some("https://www.instagram.com/reel/abc/".to_owned());
        let (id, replayed) = store
            .add("cook", 10, url.clone(), None, candidate("Soup"))
            .unwrap();
        assert!(!replayed);
        assert_eq!(
            store.add("cook", 99, url, None, candidate("Soup")).unwrap(),
            (id.clone(), true)
        );
        let mut reopened = Store::open(&dir).unwrap();
        assert_eq!(reopened.list().len(), 1);
        assert_eq!(reopened.find(&id).unwrap().received_at, 10);
        assert_eq!(
            reopened.remove("suggestion-missing"),
            Err("suggestion_not_found")
        );
        reopened.remove(&id).unwrap();
        assert!(Store::open(&dir).unwrap().list().is_empty());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn empty_unsafe_or_excess_suggestions_are_rejected_without_a_write() {
        let dir = directory("reject");
        let mut store = Store::open(&dir).unwrap();
        let add = |store: &mut Store, url: Option<&str>, original| {
            store.add("cook", 1, url.map(str::to_owned), None, original)
        };
        assert_eq!(
            add(&mut store, None, Candidate::default()),
            Err("suggestion_invalid")
        );
        for url in [
            "http://example.org/r",
            "https://user:pw@example.org/r",
            "file:///etc",
        ] {
            assert_eq!(
                add(&mut store, Some(url), candidate("A")),
                Err("suggestion_invalid")
            );
        }
        assert_eq!(
            store.add("cook", 1, None, Some("x\u{0}".to_owned()), candidate("A")),
            Err("suggestion_invalid")
        );
        assert!(!dir.join("suggestions.json").exists());
        for n in 0..CAPACITY {
            add(&mut store, None, candidate(&format!("Recipe {n}"))).unwrap();
        }
        assert_eq!(
            add(&mut store, None, candidate("One more")),
            Err("suggestion_capacity")
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
