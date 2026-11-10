use super::{
    delivery::{self, Outcome},
    time,
};
use crate::household::{self, Shared};
use axum::{
    Extension, Json,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Clone)]
pub(crate) struct WorkerCall;
#[derive(Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Operation {
    Claim,
    Settle {
        id: String,
        attempted_at: i64,
        outcome: Outcome,
    },
}
pub(crate) async fn dispatch(
    State(state): State<Shared>,
    headers: HeaderMap,
    worker: Option<Extension<WorkerCall>>,
    body: Result<Json<Operation>, JsonRejection>,
) -> Response {
    if worker.is_none() || !household::can_write(&state, &headers) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"cooking_worker_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return household::reply(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"code":"cooking_request_invalid"}),
        );
    };
    match tokio::task::spawn_blocking(move || {
        let mut store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
        let now = time::now()?;
        store.worker_checked_at = Some(now);
        let mut jobs = vec![];
        let result = store.change(None, |data| match input {
            Operation::Claim => {
                jobs = delivery::claim(data, now)?;
                Ok(())
            }
            Operation::Settle {
                id,
                attempted_at,
                outcome,
            } => delivery::settle(data, &id, attempted_at, outcome, now),
        });
        store.worker_error = result.as_ref().err().copied();
        result?;
        Ok::<_, &'static str>(json!({"jobs":jobs}))
    })
    .await
    {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => household::reply(StatusCode::SERVICE_UNAVAILABLE, json!({"code":code})),
        Err(_) => household::reply(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"code":"cooking_storage_failed"}),
        ),
    }
}
