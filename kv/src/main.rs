#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    warnings,
    trivial_numeric_casts,
    unstable_features,
    unused,
    future_incompatible
)]

use actix_web::server;
use anyhow::Result;
use log::info;

pub fn main() -> Result<()> {
    log4rs::init_file("log.yaml", Default::default())?;
    let pool = kv::db_setup()?;
    let client = reqwest::Client::new();
    let app_state = kv::AppState::new(pool, client.clone())?;
    server::new(move || kv::app_router(app_state.clone()).unwrap().finish())
        .bind("0.0.0.0:8001")?
        .run();
    kv::register(&client)?;
    info!("Potboiler-kv booted");
    Ok(())
}
