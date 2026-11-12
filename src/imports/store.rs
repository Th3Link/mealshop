use super::model::{Candidate, Method};
use serde::Serialize;
use std::fmt::Write;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    time::{Duration, Instant},
};
#[derive(Clone, Serialize)]
pub(super) struct PreviewData {
    pub(super) requested_url: String,
    pub(super) final_url: Option<String>,
    pub(super) method: Method,
    pub(super) retrieved_at: Option<i64>,
    pub(super) candidates: Vec<Candidate>,
}
pub(super) struct Pending {
    pub(super) actor: String,
    pub(super) deadline: Instant,
    pub(super) data: PreviewData,
}
#[derive(Default)]
pub(crate) struct Store {
    pub(super) previews: BTreeMap<String, Pending>,
    active: BTreeSet<String>,
    recent: BTreeMap<String, Instant>,
    pub(super) expiry_started: bool,
}
impl Store {
    pub(super) fn begin(&mut self, actor: &str) -> Result<(), &'static str> {
        let now = Instant::now();
        self.recent
            .retain(|_, time| now.duration_since(*time) < Duration::from_secs(10));
        if self.active.contains(actor) || self.recent.contains_key(actor) {
            return Err("import_fetch_cooldown");
        }
        self.active.insert(actor.to_owned());
        self.recent.insert(actor.to_owned(), now);
        Ok(())
    }
    pub(super) fn finish(&mut self, actor: &str) {
        self.active.remove(actor);
    }
    pub(super) fn expire(&mut self, now: Instant) {
        self.previews.retain(|_, p| p.deadline > now);
    }
    pub(super) fn stage(
        &mut self,
        actor: String,
        data: PreviewData,
    ) -> Result<String, &'static str> {
        self.expire(Instant::now());
        if self.previews.len() >= 16
            || self.previews.values().filter(|p| p.actor == actor).count() >= 4
        {
            return Err("import_preview_capacity");
        }
        let mut bytes = [0u8; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut bytes))
            .map_err(|_| "import_storage_failed")?;
        let mut id = String::from("import-");
        for b in bytes {
            write!(&mut id, "{b:02x}").map_err(|_| "import_storage_failed")?;
        }
        self.previews.insert(
            id.clone(),
            Pending {
                actor,
                deadline: Instant::now() + Duration::from_secs(1800),
                data,
            },
        );
        Ok(id)
    }
}
