//! Runs the Mealshop HTTP server.

use std::{error::Error, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os("APP_SECRET").is_some() {
        return run_exapp().await;
    }
    let address: SocketAddr = std::env::var("MEALSHOP_LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
        .parse()?;
    if !address.ip().is_loopback() {
        return Err("Recipe storage currently requires a loopback listen address; authentication is not implemented".into());
    }
    let json = match std::env::var("MEALSHOP_CATALOGUE_PATH") {
        Ok(path) => {
            use std::io::Read;
            let mut json = String::new();
            std::fs::File::open(path)?
                .take(1_048_577)
                .read_to_string(&mut json)?;
            json
        }
        Err(std::env::VarError::NotPresent) => include_str!("../data/catalogue.json").to_owned(),
        Err(error) => return Err(error.into()),
    };
    let directory = match std::env::var("MEALSHOP_RECIPE_DIR") {
        Ok(path) if !path.is_empty() => path,
        Ok(_) => return Err("MEALSHOP_RECIPE_DIR must not be empty".into()),
        Err(std::env::VarError::NotPresent) => "var/recipes".to_owned(),
        Err(error) => return Err(error.into()),
    };
    let listener = tokio::net::TcpListener::bind(address).await?;
    let app = if let Ok(token) = std::env::var("MEALSHOP_HOUSEHOLD_TOKEN") {
        let household_directory =
            std::env::var("MEALSHOP_HOUSEHOLD_DIR").unwrap_or_else(|_| "var/household".to_owned());
        mealshop_server::family_router(
            &json,
            directory,
            household_directory,
            token,
            std::env::var("MEALSHOP_HOUSEHOLD_READ_TOKEN").ok(),
        )?
    } else {
        mealshop_server::router_with_recipe_directory(&json, directory)?
    };
    eprintln!("Mealshop listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

async fn run_exapp() -> Result<(), Box<dyn Error>> {
    let config = mealshop_server::exapp::Config {
        secret: std::env::var("APP_SECRET")?,
        nextcloud_url: std::env::var("NEXTCLOUD_URL")?,
        directory: std::env::var("APP_PERSISTENT_STORAGE")?.into(),
        assets: std::env::var("MEALSHOP_ASSET_DIR")
            .unwrap_or_else(|_| "clients/nextcloud/dist".to_owned())
            .into(),
    };
    let app = mealshop_server::exapp::router(config).await?;
    #[cfg(unix)]
    if std::env::var_os("HP_SHARED_KEY").is_some() {
        use std::os::unix::fs::{FileTypeExt, PermissionsExt};
        let path = std::path::Path::new("/tmp/exapp.sock");
        if let Ok(metadata) = std::fs::symlink_metadata(path) {
            if !metadata.file_type().is_socket() {
                return Err("Refusing to replace a non-socket /tmp/exapp.sock".into());
            }
            std::fs::remove_file(path)?;
        }
        let listener = tokio::net::UnixListener::bind(path)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o666))?;
        eprintln!("Mealshop AppAPI unix transport ready");
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown())
            .await?;
        return Ok(());
    }
    let address = SocketAddr::new(
        std::env::var("APP_HOST")?.parse()?,
        std::env::var("APP_PORT")?.parse()?,
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("Mealshop AppAPI transport ready");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}
