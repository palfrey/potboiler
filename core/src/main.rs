#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    warnings,
    trivial_numeric_casts,
    unstable_features,
    unused,
    future_incompatible
)]

use error_chain::{
    // FIXME: Need https://github.com/rust-lang-nursery/error-chain/pull/253
    error_chain,
    error_chain_processing,
    impl_error_chain_kind,
    impl_error_chain_processed,
    impl_extract_backtrace,
    quick_main,
};
use iron;
use iron::Iron;
use log::info;
use log4rs;
use persistent;
use persistent::Read as PRead;
use potboiler;
use potboiler_common;
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
    let mut chain = potboiler::app_router(pool)?;
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    info!("Potboiler booted");
    Iron::new(chain)
        .http("0.0.0.0:8000")
        .map_err(|e| Error::with_chain(e, ErrorKind::IronError))?;
    Ok(())
});
