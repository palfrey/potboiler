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
use error_chain::{
    // FIXME: Need https://github.com/rust-lang-nursery/error-chain/pull/253
    error_chain,
    error_chain_processing,
    impl_error_chain_kind,
    impl_error_chain_processed,
    impl_extract_backtrace,
    quick_main,
};
use log::info;
use log4rs;
use potboiler;
use potboiler_common::server_id;

error_chain! {
    errors {
        IronError
    }
    links {
        PotboilerError(potboiler::Error, potboiler::ErrorKind);
    }
    foreign_links {
        LogError(log4rs::Error);
    }
}

quick_main!(|| -> Result<()> {
    log4rs::init_file("log.yaml", Default::default())?;
    let pool = potboiler::db_setup()?;
    let app_state = potboiler::AppState::new(pool, server_id::setup())?;
    info!("Potboiler booted");
    server::new(move || potboiler::app_router(app_state.clone()).unwrap().finish())
        .bind("0.0.0.0:8000")
        .unwrap()
        .run();
    Ok(())
});
