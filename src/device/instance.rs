use super::{Result, USER, random_id};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;
use std::path::Path;
use tokio::runtime::Runtime;
use tower::ServiceExt;

const MAX_BODY: usize = 8 * 1024 * 1024;

#[derive(Serialize)]
pub(super) struct Reply {
    pub status: u16,
    pub content_type: String,
    pub body: String,
}

/// One dataset owns its runtime so obsolete expiry tasks cannot retain its stores after restore.
pub(super) struct Instance {
    router: Option<Router>,
    runtime: Option<Runtime>,
    secret: String,
}
impl Instance {
    pub fn open(directory: &Path) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|_| "device_engine_failed")?;
        let secret = format!("{}{}", random_id()?, random_id()?);
        let router = runtime
            .block_on(crate::exapp::device_router(
                directory.to_owned(),
                secret.clone(),
                USER,
            ))
            .map_err(|_| "device_recovery_required")?;
        Ok(Self {
            router: Some(router),
            runtime: Some(runtime),
            secret,
        })
    }

    pub fn request(
        &self,
        path: &str,
        method: &str,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<Reply> {
        if !path.starts_with("/api/") || path.contains('#') || body.len() > MAX_BODY {
            return Err("device_request_invalid");
        }
        let method = Method::from_bytes(method.as_bytes()).map_err(|_| "device_request_invalid")?;
        if !matches!(
            method,
            Method::GET | Method::POST | Method::PUT | Method::DELETE | Method::HEAD
        ) {
            return Err("device_request_invalid");
        }
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("EX-APP-ID", "mealshop")
            .header(
                "AUTHORIZATION-APP-API",
                STANDARD.encode(format!("{USER}:{}", self.secret)),
            )
            .header("X-Mealshop-Request", "1");
        if let Some(mime) = content_type {
            builder = builder.header("Content-Type", mime);
        }
        let request = builder
            .body(Body::from(body.to_vec()))
            .map_err(|_| "device_request_invalid")?;
        let router = self.router.as_ref().ok_or("device_engine_failed")?.clone();
        self.runtime
            .as_ref()
            .ok_or("device_engine_failed")?
            .block_on(async {
                let response = router
                    .oneshot(request)
                    .await
                    .map_err(|_| "device_engine_failed")?;
                let status = response.status().as_u16();
                let content_type = response
                    .headers()
                    .get("Content-Type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("application/octet-stream")
                    .to_owned();
                let bytes = to_bytes(response.into_body(), MAX_BODY)
                    .await
                    .map_err(|_| "device_response_too_large")?;
                Ok(Reply {
                    status,
                    content_type,
                    body: STANDARD.encode(bytes),
                })
            })
    }

    fn json(&self, path: &str) -> Result<serde_json::Value> {
        let reply = self.request(path, "GET", &[], None)?;
        if reply.status != 200 {
            return Err("device_engine_failed");
        }
        let bytes = STANDARD
            .decode(reply.body)
            .map_err(|_| "device_engine_failed")?;
        serde_json::from_slice(&bytes).map_err(|_| "device_engine_failed")
    }
    pub fn households(&self) -> Result<usize> {
        self.json("/api/households")?["households"]
            .as_array()
            .map(Vec::len)
            .ok_or("device_engine_failed")
    }
    pub fn check_backup_read(&self, directory: &Path) -> Result<()> {
        let value = self.json("/api/households")?;
        for home in value["households"]
            .as_array()
            .ok_or("device_engine_failed")?
        {
            let id = home["id"].as_str().ok_or("device_engine_failed")?;
            let caps = self.json(&format!(
                "/api/households/{id}/domain/api/household/people/access"
            ))?;
            let readable = caps["people"]
                .as_array()
                .ok_or("device_engine_failed")?
                .iter()
                .filter(|person| person["can_read"] == true)
                .filter_map(|person| person["person_id"].as_str().map(str::to_owned))
                .collect();
            crate::household::people::check_backup_read(
                &directory.join(id).join("household/profiles.json"),
                &readable,
            )?;
        }
        Ok(())
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        drop(self.router.take());
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_timeout(std::time::Duration::from_secs(5));
        }
    }
}
