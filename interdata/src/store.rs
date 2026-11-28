//! Append-only file buffer for submissions. One JSON record per submission; nothing is edited.
use serde::{Deserialize, Serialize};
use std::{
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

/// Upper bound of one stored record; uploads are limited well below it.
const MAX_RECORD_BYTES: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Kind {
    Recipe,
    Ingredient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Status {
    Received,
}

/// A buffered upload. `original` is the request body exactly as received.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub(crate) schema_version: u32,
    pub(crate) id: String,
    pub(crate) kind: Kind,
    pub(crate) status: Status,
    pub(crate) received_at: u64,
    pub(crate) idempotency_key: String,
    pub(crate) original: String,
}

#[derive(Debug)]
pub(crate) enum Error {
    /// The idempotency key was used for a different upload.
    Conflict,
    Full,
    Busy,
    NotFound,
    Storage,
}

pub(crate) enum Outcome {
    Created(Record),
    Replayed(Record),
}

pub(crate) struct Limits {
    pub(crate) capacity: usize,
    pub(crate) uploads_per_minute: u32,
}

pub(crate) struct Store {
    submissions: PathBuf,
    keys: PathBuf,
    limits: Limits,
    count: usize,
    window: (u64, u32),
}

pub(crate) fn valid_id(id: &str) -> bool {
    id.strip_prefix("sub-")
        .is_some_and(|hex| hex.len() == 32 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Client-chosen replay key; restricted so that it is safe as a file name.
pub(crate) fn valid_key(key: &str) -> bool {
    (16..=128).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn new_id() -> io::Result<String> {
    let mut bytes = [0u8; 16];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let mut id = String::from("sub-");
    for b in bytes {
        write!(&mut id, "{b:02x}").map_err(io::Error::other)?;
    }
    Ok(id)
}

fn read(path: &Path) -> io::Result<String> {
    let mut text = String::new();
    File::open(path)?
        .take(MAX_RECORD_BYTES + 1)
        .read_to_string(&mut text)?;
    if text.len() as u64 > MAX_RECORD_BYTES {
        return Err(io::Error::other("record too large"));
    }
    Ok(text)
}

/// Writes a file that must not exist yet and makes it durable before returning.
fn write_new(path: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(content)?;
    file.sync_all()
}

impl Store {
    pub(crate) fn open(root: &Path, limits: Limits) -> io::Result<Self> {
        let submissions = root.join("submissions");
        let keys = root.join("keys");
        fs::create_dir_all(&submissions)?;
        fs::create_dir_all(&keys)?;
        let count = fs::read_dir(&submissions)?.count();
        Ok(Self {
            submissions,
            keys,
            limits,
            count,
            window: (0, 0),
        })
    }

    fn load(&self, id: &str) -> Result<Record, Error> {
        if !valid_id(id) {
            return Err(Error::NotFound);
        }
        let text = match read(&self.submissions.join(format!("{id}.json"))) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(Error::NotFound),
            Err(_) => return Err(Error::Storage),
        };
        serde_json::from_str::<Record>(&text)
            .ok()
            .filter(|r| r.id == id)
            .ok_or(Error::Storage)
    }

    pub(crate) fn get(&self, id: &str) -> Result<Record, Error> {
        self.load(id)
    }

    /// Stores an upload once. The same key with the same kind and body returns the first
    /// record; replays are not counted against the rate limit or capacity.
    pub(crate) fn submit(
        &mut self,
        kind: Kind,
        key: &str,
        original: &str,
        now: u64,
    ) -> Result<Outcome, Error> {
        let key_path = self.keys.join(key);
        match read(&key_path) {
            Ok(id) => {
                let record = self.load(id.trim())?;
                return if record.kind == kind && record.original == original {
                    Ok(Outcome::Replayed(record))
                } else {
                    Err(Error::Conflict)
                };
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Storage),
        }
        if self.count >= self.limits.capacity {
            return Err(Error::Full);
        }
        let minute = now / 60;
        if self.window.0 != minute {
            self.window = (minute, 0);
        }
        if self.window.1 >= self.limits.uploads_per_minute {
            return Err(Error::Busy);
        }
        let record = Record {
            schema_version: 1,
            id: new_id().map_err(|_| Error::Storage)?,
            kind,
            status: Status::Received,
            received_at: now,
            idempotency_key: key.to_owned(),
            original: original.to_owned(),
        };
        let json = serde_json::to_vec(&record).map_err(|_| Error::Storage)?;
        // The record is written before its key: a crash in between leaves an unreferenced
        // record, never a key that points at nothing.
        write_new(&self.submissions.join(format!("{}.json", record.id)), &json)
            .and_then(|()| write_new(&key_path, record.id.as_bytes()))
            .map_err(|_| Error::Storage)?;
        self.count += 1;
        self.window.1 += 1;
        Ok(Outcome::Created(record))
    }
}
