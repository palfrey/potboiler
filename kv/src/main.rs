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
    quick_main,
};

use kv;
use potboiler_common::{http_client, server_id};
use iron::{self, prelude::*};
use log::{info};
use persistent::{Read as PRead};

quick_main!(|| -> kv::Result<()> {
    log4rs::init_file("log.yaml", Default::default())?;
    let pool = kv::db_setup()?;
    let mut router = kv::app_router(pool)?;
    router.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    let client = reqwest::Client::new();
    kv::register(&client)?;
    http_client::set_client(&mut router, client);
    info!("Potboiler-kv booted");
    Iron::new(router).http("0.0.0.0:8001").unwrap();
    Ok(())
});