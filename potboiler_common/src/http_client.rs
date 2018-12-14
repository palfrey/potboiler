use hyper;
use iron::{self, typemap::Key};
use persistent::Read as PRead;

#[derive(Clone, Copy, Debug)]
pub struct HyperKey;

impl Key for HyperKey {
    type Value = hyper::Client;
}

#[macro_export]
macro_rules! get_http_client {
    ($req:expr) => {
        match $req.extensions.get::<persistent::Read<http_client::HyperKey>>() {
            Some(client) => client,
            None => {
                println!("Couldn't get the http client from the request!");
                return Ok(Response::with(status::InternalServerError));
            }
        }
    };
}

pub fn set_client(router: &mut iron::Chain, client: hyper::Client) {
    router.link(PRead::<HyperKey>::both(client));
}
