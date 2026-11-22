#![allow(clippy::unwrap_used)]

use super::{client, config};
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned, pki_types};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::Arc,
    thread,
    time::Duration,
};

const ROOT: &[u8] = include_bytes!("../tests/fixtures/tls/root.der");
const VALID: &[u8] = include_bytes!("../tests/fixtures/tls/localhost.der");
const EXPIRED: &[u8] = include_bytes!("../tests/fixtures/tls/expired.der");
const KEY: &[u8] = include_bytes!("../tests/fixtures/tls/test-only-key.der");

fn trusted_client() -> reqwest::ClientBuilder {
    let mut roots = RootCertStore::empty();
    roots.add(pki_types::CertificateDer::from(ROOT)).unwrap();
    reqwest::Client::builder().tls_backend_preconfigured(config(roots).unwrap())
}

async fn request(
    client: reqwest::ClientBuilder,
    certificate: &'static [u8],
    host: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    let server =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(
                vec![pki_types::CertificateDer::from(certificate)],
                pki_types::PrivatePkcs8KeyDer::from(KEY.to_vec()).into(),
            )
            .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let worker = thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut tls = StreamOwned::new(ServerConnection::new(Arc::new(server)).unwrap(), socket);
        let mut buffer = [0_u8; 4096];
        if tls.read(&mut buffer).is_ok_and(|size| size > 0) {
            let _ =
                tls.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            let _ = tls.flush();
        }
    });
    let response = client
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
        .get(format!("https://{host}:{port}/"))
        .send()
        .await;
    worker.join().unwrap();
    response
}

#[tokio::test]
async fn configured_tls_accepts_a_trusted_matching_certificate() {
    assert_eq!(
        request(trusted_client(), VALID, "localhost")
            .await
            .unwrap()
            .status(),
        200
    );
}

#[tokio::test]
async fn production_roots_reject_a_private_test_authority() {
    let error = request(client().unwrap(), VALID, "localhost")
        .await
        .unwrap_err();
    assert!(error.is_connect());
}

#[tokio::test]
async fn trusted_authority_does_not_bypass_hostname_validation() {
    let error = request(trusted_client(), VALID, "127.0.0.1")
        .await
        .unwrap_err();
    assert!(error.is_connect());
}

#[tokio::test]
async fn trusted_authority_does_not_bypass_expiry_validation() {
    let error = request(trusted_client(), EXPIRED, "localhost")
        .await
        .unwrap_err();
    assert!(error.is_connect());
}
