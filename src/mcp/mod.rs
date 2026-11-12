//! Explicitly scoped recipe tools over the existing authenticated household boundary.
mod api;
mod protocol;
mod schemas;
mod store;
mod tools;
pub(crate) use crate::household::Authority;
use crate::membership::Role;
pub(crate) use api::routes;
use serde::{Deserialize, Serialize};
pub(crate) use store::Store;
#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Scope {
    #[default]
    Disabled,
    Read,
    Create,
}
impl Scope {
    fn effective(self, role: Role) -> Self {
        if self == Self::Create && role == Role::Reader {
            Self::Read
        } else {
            self
        }
    }
}

fn maintain_previews(state: &crate::household::Shared) -> Result<(), &'static str> {
    let mut store = state.mcp.lock().map_err(|_| "mcp_storage_failed")?;
    if store.expiry_started {
        return Ok(());
    }
    store.expiry_started = true;
    drop(store);
    let weak = std::sync::Arc::downgrade(state);
    std::mem::drop(tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            {
                let Some(state) = weak.upgrade() else {
                    break;
                };
                if let Ok(mut store) = state.mcp.lock() {
                    store.expire(std::time::Instant::now());
                }
            }
        }
    }));
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod store_tests;
