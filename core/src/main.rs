#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
extern crate error_chain;
extern crate iron;
extern crate persistent;
extern crate potboiler;
extern crate potboiler_common;

use iron::Iron;
use persistent::Read as PRead;
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
