//! Runs the Mealshop HTTP server.

use std::{error::Error, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address: SocketAddr = std::env::var("MEALSHOP_LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("Mealshop listening on {}", listener.local_addr()?);
    axum::serve(listener, mealshop_server::router())
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("Cannot install shutdown signal handler: {error}");
    }
}
