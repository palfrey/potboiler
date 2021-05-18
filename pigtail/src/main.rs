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
use potboiler_common::pg;
use std::env;

fn main() -> Result<()> {
    log4rs::init_file("log.yaml", Default::default()).expect("log config ok");
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = pg::get_pool(db_url).unwrap();
    let app_state = pigtail::AppState::new(pool)?;
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap();
    server::new(move || pigtail::app_router(app_state.clone()).unwrap().finish())
        .bind(("0.0.0.0", port))
        .unwrap()
        .run();
    pigtail::register();
    info!("Pigtail booted");
    Ok(())
}
