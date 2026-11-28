//! Runs the Interdata intake service.
use std::{env, error::Error, net::SocketAddr};

fn optional(name: &str) -> Result<Option<String>, Box<dyn Error>> {
    match env::var(name) {
        Ok(value) if value.is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(format!("{name}: {error}").into()),
    }
}

fn number<T: std::str::FromStr>(name: &str, default: T) -> Result<T, Box<dyn Error>> {
    match optional(name)? {
        Some(value) => value
            .parse()
            .map_err(|_| format!("{name} must be a whole number").into()),
        None => Ok(default),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address: SocketAddr = optional("INTERDATA_LISTEN_ADDR")?
        .unwrap_or_else(|| "127.0.0.1:3100".to_owned())
        .parse()?;
    let app = mealshop_interdata::router(mealshop_interdata::Config {
        upload_secret: optional("INTERDATA_UPLOAD_SECRET")?
            .ok_or("INTERDATA_UPLOAD_SECRET is required")?,
        data_dir: optional("INTERDATA_DATA_DIR")?
            .unwrap_or_else(|| "var/interdata".to_owned())
            .into(),
        cdn_origin: optional("INTERDATA_CDN_ORIGIN")?,
        capacity: number("INTERDATA_CAPACITY", 10_000)?,
        uploads_per_minute: number("INTERDATA_UPLOADS_PER_MINUTE", 60)?,
    })?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("Interdata listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    #[cfg(unix)]
    {
        if let Ok(mut terminate) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}
