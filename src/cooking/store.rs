use super::{
    model::{self, Data, Run, RunState},
    plan::{self, Proposal},
    time,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize)]
pub(crate) struct Prepared {
    #[serde(skip)]
    pub(crate) actor: String,
    pub(crate) preview_id: String,
    pub(crate) revision: u64,
    pub(crate) expires_at: i64,
    pub(crate) run: Run,
    pub(crate) previous: Option<Run>,
    pub(crate) proposal: Proposal,
}
pub(crate) struct Store {
    path: PathBuf,
    pub(crate) data: Data,
    writable: bool,
    pub(crate) worker_checked_at: Option<i64>,
    pub(crate) worker_error: Option<&'static str>,
    previews: BTreeMap<String, Prepared>,
    next_preview: u64,
    preview_epoch: u128,
}
impl Store {
    pub(crate) fn health(&self) -> serde_json::Value {
        serde_json::json!({"last_check_at":self.worker_checked_at,"error":self.worker_error,"writable":self.writable})
    }

    pub(crate) fn open(directory: &Path) -> Result<Self, String> {
        if directory.join(".cooking.pending").exists() {
            return Err(
                "Interrupted cooking write; restore a verified household backup before startup"
                    .to_owned(),
            );
        }
        let path = directory.join("cooking.json");
        let data = if path.exists() {
            let mut bytes = Vec::new();
            File::open(&path)
                .map_err(|_| "Cannot read cooking data")?
                .take(4_194_305)
                .read_to_end(&mut bytes)
                .map_err(|_| "Cannot read cooking data")?;
            if bytes.len() > 4_194_304 {
                return Err("Cooking data exceeds 4 MiB".to_owned());
            }
            serde_json::from_slice(&bytes)
                .map_err(|_| "Invalid cooking data; restore a verified backup")?
        } else {
            Data::default()
        };
        validate(&data).map_err(|_| "Invalid cooking data; restore a verified backup")?;
        let mut store = Self {
            path,
            data,
            writable: true,
            worker_checked_at: None,
            worker_error: None,
            previews: BTreeMap::new(),
            next_preview: 1,
            preview_epoch: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| "Invalid system clock")?
                .as_nanos(),
        };
        store
            .change(None, |data| {
                for notice in &mut data.notices {
                    if let super::delivery::Delivery::Sending { attempted_at } = notice.delivery {
                        notice.delivery = super::delivery::Delivery::Unknown { attempted_at };
                    }
                }
                Ok(())
            })
            .map_err(|_| "Cannot recover interrupted notification attempts")?;
        Ok(store)
    }
    pub(crate) fn change(
        &mut self,
        expected: Option<u64>,
        action: impl FnOnce(&mut Data) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if !self.writable {
            return Err("cooking_storage_failed");
        }
        if expected.is_some_and(|value| value != self.data.revision) {
            return Err("cooking_revision_conflict");
        }
        let mut next = self.data.clone();
        action(&mut next)?;
        if next == self.data {
            return Ok(());
        }
        if expected.is_some() {
            next.revision = next.revision.checked_add(1).ok_or("cooking_capacity")?;
        } else {
            next.delivery_revision = next
                .delivery_revision
                .checked_add(1)
                .ok_or("cooking_capacity")?;
        }
        validate(&next)?;
        let bytes = serde_json::to_vec(&next).map_err(|_| "cooking_storage_failed")?;
        if bytes.len() > 4_194_304 {
            return Err("cooking_capacity");
        }
        let pending = self.path.with_file_name(".cooking.pending");
        let persist = || -> std::io::Result<()> {
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
        if persist().is_err() {
            self.writable = false;
            return Err("cooking_storage_failed");
        }
        self.data = next;
        Ok(())
    }
    pub(crate) fn prepare(
        &mut self,
        actor: String,
        mut run: Run,
        previous: Option<Run>,
        proposal: Proposal,
        now: i64,
    ) -> Result<Prepared, &'static str> {
        self.previews
            .retain(|_, p| p.expires_at >= now && p.actor != actor);
        if self.previews.len() >= 20 {
            return Err("cooking_preview_capacity");
        }
        let preview_id = format!("preview-{}-{}", self.preview_epoch, self.next_preview);
        self.next_preview = self.next_preview.checked_add(1).ok_or("cooking_capacity")?;
        run.schedule.clone_from(&proposal.schedule);
        let value = Prepared {
            actor,
            preview_id: preview_id.clone(),
            revision: self.data.revision,
            expires_at: time::add(now, 600)?,
            run,
            previous,
            proposal,
        };
        self.previews.insert(preview_id, value.clone());
        Ok(value)
    }
    pub(crate) fn apply(
        &mut self,
        actor: &str,
        id: &str,
        expected: u64,
        now: i64,
    ) -> Result<String, &'static str> {
        let prepared = self
            .previews
            .get(id)
            .filter(|p| p.actor == actor && p.expires_at >= now)
            .ok_or("cooking_preview_expired")?
            .clone();
        if prepared.revision != expected {
            return Err("cooking_revision_conflict");
        }
        let mut run = prepared.run;
        let mut result = run.id.clone();
        self.change(Some(expected), |data| {
            if prepared.previous.is_some() {
                let id = run.id.clone();
                *data.run_mut(&id)? = run;
            } else {
                run.id = data.allocate("cook")?;
                result.clone_from(&run.id);
                data.runs.push(run);
            }
            super::delivery::cancel_obsolete(data);
            Ok(())
        })?;
        self.previews.remove(id);
        Ok(result)
    }
}
fn validate(data: &Data) -> Result<(), &'static str> {
    if data.schema_version != 1
        || data.next_id == 0
        || data.runs.len() > 100
        || data.notices.len() > 2000
    {
        return Err("cooking_capacity");
    }
    let mut ids = BTreeSet::new();
    for run in &data.runs {
        if !ids.insert(&run.id)
            || !allocated(data, &run.id, "cook-")
            || !model::text(&run.name, 200)
            || !crate::recipes::valid_id(&run.recipe.id)
            || run.recipe.revision == 0
            || !model::text(&run.recipe.title, 200)
            || !matches!(run.recipe.language.as_str(), "en" | "de")
            || !(1..=1_000_000).contains(&run.servings)
            || !(1..=1_000_000).contains(&run.recipe.reference_servings)
            || !time::valid(run.created_at)
            || time::zone(&run.timezone).is_err()
            || run
                .target
                .as_ref()
                .is_some_and(|t| t.timezone != run.timezone || t.resolve().is_err())
            || run.subscriptions.len() > 100
        {
            return Err("cooking_data_invalid");
        }
        let stages: Vec<_> = run.steps.iter().map(|s| s.stage.clone()).collect();
        plan::order(&stages)?;
        for step in &run.steps {
            step.stage.validate()?;
            if !model::timer_valid(&step.timer)
                || step.not_before.is_some_and(|at| !time::valid(at))
                || step.generation == 0
                || step.cycle == 0
                || step.cycle > step.stage.repeat
                || step.lag_seconds > 604_800
            {
                return Err("cooking_data_invalid");
            }
        }
        if (run.status == RunState::Completed && !run.steps.iter().all(|s| s.timer.complete()))
            || run.schedule.len() != run.steps.len()
        {
            return Err("cooking_data_invalid");
        }
        let mut timing_ids = BTreeSet::new();
        for t in &run.schedule {
            if !timing_ids.insert(&t.id)
                || !run.steps.iter().any(|s| s.stage.id == t.id)
                || ![t.start, t.active_until, t.finish, t.earliest, t.latest]
                    .into_iter()
                    .all(time::valid)
                || t.start > t.active_until
                || t.active_until > t.finish
                || t.earliest > t.start
                || t.start > t.latest
            {
                return Err("cooking_data_invalid");
            }
        }
        for (user, sub) in &run.subscriptions {
            if !super::actor_valid(user)
                || !matches!(sub.language.as_str(), "en" | "de")
                || !time::valid(sub.reported_at)
            {
                return Err("cooking_data_invalid");
            }
        }
    }
    super::delivery::validate(data)
}
pub(crate) fn allocated(data: &Data, id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix)
        .and_then(|s| s.parse::<u64>().ok())
        .is_some_and(|n| n > 0 && n < data.next_id && id == format!("{prefix}{n}"))
}
