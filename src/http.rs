//! Keep outbound TLS consistent across the server and the Android core.
use rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;

fn config(roots: RootCertStore) -> Result<ClientConfig, &'static str> {
    Ok(
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|_| "Cannot initialize outbound TLS protocol versions")?
            .with_root_certificates(roots)
            .with_no_client_auth(),
    )
}

pub(crate) fn client() -> Result<reqwest::ClientBuilder, &'static str> {
    let roots = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    Ok(reqwest::Client::builder().tls_backend_preconfigured(config(roots)?))
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
