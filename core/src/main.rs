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
use failure::Error;
use log::info;
use potboiler_common::server_id;

pub fn main() -> Result<(), Error> {
    log4rs::init_file("log.yaml", Default::default())?;
    let pool = potboiler::db_setup()?;
    let app_state = potboiler::AppState::new(pool, server_id::setup())?;
    server::new(move || potboiler::app_router(app_state.clone()).unwrap().finish())
        .bind("0.0.0.0:8000")
        .unwrap()
        .run();
    info!("Potboiler booted");
    Ok(())
}
