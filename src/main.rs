//! Runs the Mealshop HTTP server.

use std::{error::Error, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    let mut app = mealshop_server::router_with_recipe_directory(&json, directory)?;
    if let Ok(token) = std::env::var("MEALSHOP_HOUSEHOLD_TOKEN") {
        let directory =
            std::env::var("MEALSHOP_HOUSEHOLD_DIR").unwrap_or_else(|_| "var/household".to_owned());
        app = app.merge(mealshop_server::household_router(&json, directory, token)?);
    }
    eprintln!("Mealshop listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("Cannot install shutdown signal handler: {error}");
    }
}
