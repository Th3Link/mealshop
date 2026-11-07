//! Checks startup errors without requiring external services.
#![allow(clippy::unwrap_used)]

use std::{net::TcpListener, process::Command};

#[test]
fn invalid_address_exits_with_an_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_mealshop-server"))
        .env("MEALSHOP_LISTEN_ADDR", "invalid-address")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stderr.starts_with(b"Error: "));
}

#[test]
fn occupied_port_exits_with_an_error() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_mealshop-server"))
        .env(
            "MEALSHOP_LISTEN_ADDR",
            listener.local_addr().unwrap().to_string(),
        )
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stderr.starts_with(b"Error: "));
}

#[test]
fn invalid_or_missing_catalogue_prevents_startup() {
    for path in ["Cargo.toml", "tests/fixtures/nonexistent.json"] {
        let output = Command::new(env!("CARGO_BIN_EXE_mealshop-server"))
            .env(
                "MEALSHOP_CATALOGUE_PATH",
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path),
            )
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stderr.starts_with(b"Error: "));
    }
}
