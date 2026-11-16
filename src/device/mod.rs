//! Durable on-device execution through private standard I/O; no network listener is opened.
mod backup;
mod dataset;
mod instance;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    fmt::Write as _,
    io::{BufRead, Read, Write},
    path::Path,
};

type Result<T> = std::result::Result<T, &'static str>;
const USER: &str = "device";
const MAX_FRAME: u64 = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    id: u64,
    command: Command,
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Command {
    Status,
    Backup {
        passphrase: String,
    },
    Exported {
        id: String,
    },
    DiscardExport {
        id: String,
    },
    Restore {
        id: String,
        passphrase: String,
    },
    CommitRestore {
        token: String,
    },
    DiscardRestore {
        token: String,
    },
    DiscardImport {
        id: String,
    },
    Request {
        generation: Option<String>,
        path: String,
        method: String,
        body: Option<String>,
        content_type: Option<String>,
    },
    Shutdown,
}

struct Engine {
    active: Option<(String, instance::Instance)>,
    restore: Option<PendingRestore>,
    export: Option<(String, backup::Manifest)>,
    recovery: Option<&'static str>,
    epoch: u64,
    cleanup_complete: bool,
    // Declared last: lifetime ownership outlives all dataset runtimes and their stores.
    root: dataset::Root,
}
struct PendingRestore {
    token: String,
    id: String,
    instance: instance::Instance,
    epoch: u64,
}
impl Engine {
    fn open(path: &Path) -> Result<Self> {
        let root = dataset::Root::open(path)?;
        let transfers_clean = backup::cleanup_transfers(&root).is_ok();
        let opened = (|| {
            let id = root.active()?;
            let path = root.dataset(&id)?;
            dataset::recover_markers(&path)?;
            let instance = instance::Instance::open(&path)?;
            Ok((id, instance))
        })();
        let (active, recovery) = match opened {
            Ok(value) => (Some(value), None),
            Err(code) => (None, Some(code)),
        };
        let datasets_clean = active
            .as_ref()
            .is_none_or(|(id, _)| root.cleanup_inactive(id).is_ok());
        let cleanup_complete = transfers_clean && datasets_clean;
        Ok(Self {
            root,
            active,
            recovery,
            epoch: 0,
            restore: None,
            export: None,
            cleanup_complete,
        })
    }
    fn status(&self) -> Value {
        let last: Option<backup::Manifest> =
            dataset::read_json(&self.root.path.join("last-backup.json"))
                .ok()
                .filter(|m: &backup::Manifest| {
                    m.schema_version == 1
                        && m.kind == "mealshop-app-only"
                        && m.created_at <= backup::MAX_TIMESTAMP
                        && m.bytes <= backup::MAX_ARCHIVE
                        && m.files <= 20_000
                        && m.households <= 20_000
                });
        json!({"protocol":1,"kind":"device","engine_version":env!("CARGO_PKG_VERSION"),"source_sha256":option_env!("MEALSHOP_CORE_SOURCE"),"storage":if self.active.is_some(){"ready"}else{"recovery"},"error":self.recovery,"last_backup":last,"process":std::process::id(),"generation":self.active.as_ref().map(|(id,_)|id),"cleanup_complete":self.cleanup_complete})
    }
    fn execute(&mut self, command: Command) -> Result<Value> {
        match command {
            Command::Status => Ok(self.status()),
            Command::Backup { passphrase } => {
                if let Some((id, _)) = self.export.take() {
                    backup::discard(&self.root, "exports", &id)?;
                }
                let (id, instance) = self.active.as_ref().ok_or("device_recovery_required")?;
                instance.check_backup_read(&self.root.dataset(id)?)?;
                let (id, manifest) =
                    backup::export(&self.root, id, instance.households()?, passphrase)?;
                let value = json!({"id":id,"manifest":manifest});
                self.export = Some((id, manifest));
                Ok(value)
            }
            Command::Exported { id } => {
                let (exported, manifest) = self.export.as_ref().ok_or("device_request_invalid")?;
                if *exported != id {
                    return Err("device_request_invalid");
                }
                dataset::atomic_json(&self.root.path, "last-backup.json", manifest)?;
                backup::discard(&self.root, "exports", &id)?;
                self.export = None;
                Ok(json!({"saved":true}))
            }
            Command::DiscardExport { id } => {
                backup::discard(&self.root, "exports", &id)?;
                if self
                    .export
                    .as_ref()
                    .is_some_and(|(active, _)| *active == id)
                {
                    self.export = None;
                }
                Ok(json!({"discarded":true}))
            }
            Command::Restore { id, passphrase } => {
                self.discard_restore()?;
                let (candidate, manifest) = backup::restore(&self.root, &id, passphrase)?;
                let validated = (|| {
                    let instance = instance::Instance::open(&self.root.dataset(&candidate)?)?;
                    if instance.households()? != manifest.households {
                        return Err("device_restore_invalid");
                    }
                    Ok((random_id()?, instance))
                })();
                let (token, instance) = match validated {
                    Ok(value) => value,
                    Err(code) => {
                        self.root.remove_dataset(&candidate)?;
                        return Err(code);
                    }
                };
                let value = json!({"token":token,"manifest":manifest});
                self.restore = Some(PendingRestore {
                    token,
                    id: candidate,
                    instance,
                    epoch: self.epoch,
                });
                Ok(value)
            }
            Command::CommitRestore { token } => self.commit_restore(&token),
            Command::DiscardRestore { token } => {
                let current = self
                    .restore
                    .as_ref()
                    .is_some_and(|pending| pending.token == token);
                if current {
                    self.discard_restore()?;
                }
                Ok(json!({"discarded":current}))
            }
            Command::DiscardImport { id } => {
                backup::discard(&self.root, "imports", &id)?;
                Ok(json!({"discarded":true}))
            }
            Command::Request {
                generation,
                path,
                method,
                body,
                content_type,
            } => {
                if !matches!(method.as_str(), "GET" | "HEAD") {
                    self.epoch = self.epoch.checked_add(1).ok_or("device_capacity")?;
                }
                let (id, instance) = self.active.as_ref().ok_or("device_recovery_required")?;
                if generation.as_deref() != Some(id.as_str()) {
                    return Err("device_dataset_changed");
                }
                let body = body
                    .map(|b| STANDARD.decode(b))
                    .transpose()
                    .map_err(|_| "device_request_invalid")?
                    .unwrap_or_default();
                let reply = instance.request(&path, &method, &body, content_type.as_deref())?;
                serde_json::to_value(reply).map_err(|_| "device_engine_failed")
            }
            Command::Shutdown => Ok(json!({"stopped":true})),
        }
    }
    fn discard_restore(&mut self) -> Result<()> {
        if let Some(pending) = self.restore.take() {
            let id = pending.id.clone();
            drop(pending);
            self.root.remove_dataset(&id)?;
        }
        Ok(())
    }
    fn commit_restore(&mut self, token: &str) -> Result<Value> {
        let pending = self.restore.as_ref().ok_or("device_restore_expired")?;
        if pending.token != token || pending.epoch != self.epoch {
            return Err("device_restore_expired");
        }
        if let Err(code) = self.root.publish(&pending.id) {
            // Rename may have completed before directory fsync failed. Stop writes until reopen
            // resolves the published pointer, rather than writing to an obsolete generation.
            self.active = None;
            self.restore = None;
            self.recovery = Some(code);
            return Err(code);
        }
        let pending = self.restore.take().ok_or("device_restore_expired")?;
        let id = pending.id;
        let old = self.active.replace((id.clone(), pending.instance));
        drop(old);
        self.recovery = None;
        self.epoch = self.epoch.checked_add(1).ok_or("device_capacity")?;
        let cleanup = self.root.cleanup_inactive(&id).is_ok();
        self.cleanup_complete = cleanup;
        Ok(json!({"restored":true,"cleanup_complete":cleanup}))
    }
}

/// Runs the app's bounded private protocol until its owner closes stdin or requests shutdown.
/// # Errors
/// Returns a static error code for unavailable storage or a broken/oversized protocol stream.
pub fn run_stdio(path: &Path) -> std::result::Result<(), String> {
    run(path).map_err(str::to_owned)
}
fn run(path: &Path) -> Result<()> {
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    let mut engine = match Engine::open(path) {
        Ok(engine) => engine,
        Err(code) => {
            send(
                &mut output,
                &json!({"protocol":1,"kind":"device","error":code}),
            )?;
            return Err(code);
        }
    };
    send(&mut output, &engine.status())?;
    loop {
        let mut bytes = Vec::new();
        let length = input
            .by_ref()
            .take(MAX_FRAME + 1)
            .read_until(b'\n', &mut bytes)
            .map_err(|_| "device_protocol_failed")?;
        if length == 0 {
            return Ok(());
        }
        if u64::try_from(length).map_err(|_| "device_protocol_failed")? > MAX_FRAME {
            return Err("device_frame_too_large");
        }
        let Ok(frame) = serde_json::from_slice::<Frame>(&bytes) else {
            send(
                &mut output,
                &json!({"id":0,"ok":false,"error":"device_request_invalid"}),
            )?;
            continue;
        };
        let stop = matches!(frame.command, Command::Shutdown);
        let response = match engine.execute(frame.command) {
            Ok(value) => json!({"id":frame.id,"ok":true,"value":value}),
            Err(code) => json!({"id":frame.id,"ok":false,"error":code}),
        };
        send(&mut output, &response)?;
        if stop {
            return Ok(());
        }
    }
}
fn send(output: &mut impl Write, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *output, value).map_err(|_| "device_protocol_failed")?;
    output
        .write_all(b"\n")
        .and_then(|()| output.flush())
        .map_err(|_| "device_protocol_failed")
}
fn random_id() -> Result<String> {
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|_| "device_random_failed")?;
    let mut value = String::with_capacity(32);
    for byte in bytes {
        write!(&mut value, "{byte:02x}").map_err(|_| "device_random_failed")?;
    }
    Ok(value)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::too_many_lines)]
mod tests;
