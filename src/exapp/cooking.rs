use super::{App, Engine, Role, STANDARD, Shared};
use crate::cooking::{
    Job, Outcome,
    internal::{Operation, WorkerCall},
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, header},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tower::ServiceExt;
#[derive(Deserialize)]
struct Reply {
    jobs: Vec<Job>,
}
pub(super) fn start(app: &Shared) {
    let weak = Arc::downgrade(app);
    let task = tokio::spawn(async move {
        let mut cursor = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let Some(app) = weak.upgrade() else {
                return;
            };
            if app.enabled.load(Ordering::Acquire) {
                cursor = tick(&app, cursor).await;
            }
            drop(app);
        }
    });
    if let Ok(mut slot) = app.cooking_worker.lock() {
        *slot = Some(task.abort_handle());
    }
}
async fn tick(app: &Shared, cursor: usize) -> usize {
    let ids = match app.membership.lock() {
        Ok(registry) => registry.household_ids(),
        Err(_) => return cursor,
    };
    if ids.is_empty() {
        return 0;
    }
    let mut sent = 0;
    let mut next = cursor % ids.len();
    for offset in 0..ids.len() {
        let index = (cursor + offset) % ids.len();
        next = (index + 1) % ids.len();
        let id = ids[index].clone();
        if !app.enabled.load(Ordering::Acquire) {
            return next;
        }
        if !app
            .config
            .directory
            .join(&id)
            .join("household/cooking.json")
            .is_file()
        {
            continue;
        }
        let Ok(router) = super::household_router(app.clone(), id.clone()).await else {
            continue;
        };
        let Some(jobs) = internal(&router, &app.config.secret, Operation::Claim).await else {
            continue;
        };
        for job in jobs {
            let permitted = app.enabled.load(Ordering::Acquire)
                && app
                    .membership
                    .lock()
                    .ok()
                    .and_then(|r| r.role(&id, &job.user))
                    .is_some_and(|role| role != Role::Reader);
            let outcome = if permitted {
                send(app, &id, &job).await
            } else {
                Outcome::Revoked
            };
            sent += 1;
            if internal(
                &router,
                &app.config.secret,
                Operation::Settle {
                    id: job.id,
                    attempted_at: job.attempted_at,
                    outcome,
                },
            )
            .await
            .is_none()
            {
                eprintln!("cooking_delivery_settle_failed; review persisted notification state");
            }
        }
        if sent >= 8 {
            break;
        }
    }
    next
}
async fn internal(router: &axum::Router, secret: &str, operation: Operation) -> Option<Vec<Job>> {
    let body = serde_json::to_vec(&operation).ok()?;
    let request = Request::builder()
        .method("POST")
        .uri("/api/household/cooking/internal")
        .header(header::AUTHORIZATION, format!("Bearer {secret}"))
        .header(header::CONTENT_TYPE, "application/json")
        .extension(WorkerCall)
        .body(Body::from(body))
        .ok()?;
    let response = router.clone().oneshot(request).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let bytes = to_bytes(response.into_body(), 65_536).await.ok()?;
    serde_json::from_slice::<Reply>(&bytes)
        .ok()
        .map(|reply| reply.jobs)
}
fn notification(job: &Job, link: &str, home: &str) -> Value {
    let german = job.language == "de";
    let subject = match (german, job.event) {
        (false, crate::cooking::delivery::Event::Prepare) => "Mealshop: prepare the next action",
        (false, crate::cooking::delivery::Event::Start) => "Mealshop: planned action due",
        (false, crate::cooking::delivery::Event::ActiveEnded) => "Mealshop: active phase ended",
        (false, crate::cooking::delivery::Event::TimerEnded) => {
            "Mealshop: timer elapsed; check the result"
        }
        (true, crate::cooking::delivery::Event::Prepare) => {
            "Mealshop: nächsten Schritt vorbereiten"
        }
        (true, crate::cooking::delivery::Event::Start) => "Mealshop: geplanter Schritt fällig",
        (true, crate::cooking::delivery::Event::ActiveEnded) => "Mealshop: Arbeitsphase beendet",
        (true, crate::cooking::delivery::Event::TimerEnded) => {
            "Mealshop: Zeit abgelaufen; Ergebnis prüfen"
        }
    };
    let equipment = if job.equipment.is_empty() {
        String::new()
    } else {
        format!(
            " · {}: {}",
            if german { "Ausstattung" } else { "Equipment" },
            job.equipment.join(", ")
        )
    };
    json!({"params":{"object":"mealshop_cooking","object_id":format!("{home}-{}",job.id),"subject_type":"app_api_ex_app","subject_params":{"rich_subject":subject,"rich_subject_params":{},"rich_message":"{action}","rich_message_params":{"action":{"type":"highlight","id":job.id,"name":format!("{}{}",job.action,equipment)}},"link":link}}})
}
async fn send(app: &App, home: &str, job: &Job) -> Outcome {
    let public = app
        .config
        .public_url
        .as_ref()
        .unwrap_or(&app.config.nextcloud_url)
        .trim_end_matches('/');
    let link = format!(
        "{public}/index.php/apps/app_api/embedded/mealshop/main/h/{home}/{}/{}",
        job.destination.path(),
        job.run_id
    );
    let url = format!(
        "{}/ocs/v1.php/apps/app_api/api/v1/notification?format=json",
        app.config.nextcloud_url.trim_end_matches('/')
    );
    let response = app
        .client
        .post(url)
        .timeout(Duration::from_secs(5))
        .header("EX-APP-ID", "mealshop")
        .header("EX-APP-VERSION", env!("CARGO_PKG_VERSION"))
        .header(
            "AUTHORIZATION-APP-API",
            STANDARD.encode(format!("{}:{}", job.user, app.config.secret)),
        )
        .header("OCS-APIRequest", "true")
        .json(&notification(job, &link, home))
        .send()
        .await;
    let mut response = match response {
        Ok(r) => r,
        Err(e) => {
            return if e.is_connect() {
                Outcome::ConnectionFailed
            } else {
                Outcome::Unknown
            };
        }
    };
    let status = response.status();
    if status.as_u16() == 404 {
        return Outcome::Unavailable;
    }
    if status.is_client_error() {
        return Outcome::Rejected;
    }
    if !status.is_success() {
        return Outcome::Unknown;
    }
    if response
        .content_length()
        .is_some_and(|length| length > 65_536)
    {
        return Outcome::Unknown;
    }
    let mut bytes = vec![];
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) if bytes.len() + chunk.len() <= 65_536 => {
                bytes.extend_from_slice(&chunk);
            }
            Ok(None) => break,
            _ => return Outcome::Unknown,
        }
    }
    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return Outcome::Unknown;
    };
    if value
        .pointer("/ocs/meta/statuscode")
        .and_then(Value::as_u64)
        .is_some_and(|code| code == 100 || code == 200)
    {
        Outcome::Accepted
    } else {
        Outcome::Unknown
    }
}
