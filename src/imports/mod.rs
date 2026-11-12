//! Reviewed customer-operated public recipe import; no scripts or credentials cross this boundary.
pub(super) static PROCESSING: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(4);
mod api;
mod html;
mod model;
mod network;
mod parser;
mod store;
pub(crate) use api::routes;
pub(crate) use model::Receipt;
pub(crate) use store::Store;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod network_tests;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod parser_tests;

fn maintain(state: &crate::household::Shared) {
    if let Ok(mut store) = state.imports.lock() {
        if store.expiry_started {
            return;
        }
        store.expiry_started = true;
    } else {
        return;
    }
    let weak = std::sync::Arc::downgrade(state);
    std::mem::drop(tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let Some(state) = weak.upgrade() else {
                break;
            };
            if let Ok(mut store) = state.imports.lock() {
                store.expire(std::time::Instant::now());
            }
        }
    }));
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod receipt_tests;
