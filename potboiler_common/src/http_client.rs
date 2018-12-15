use reqwest;
use iron;
use iron::typemap::Key;
use persistent::Read as PRead;

#[derive(Clone, Copy, Debug)]
pub struct ReqwestKey;

impl Key for ReqwestKey {
    type Value = reqwest::Client;
}

#[macro_export]
macro_rules! get_http_client {
    ($req:expr) => {
        match $req.extensions.get::<persistent::Read<http_client::ReqwestKey>>() {
            Some(client) => client,
            None => {
                println!("Couldn't get the http client from the request!");
                return Ok(Response::with(status::InternalServerError));
            }
        }
    };
}

pub fn set_client(router: &mut iron::Chain, client: reqwest::Client) {
    router.link(PRead::<ReqwestKey>::both(client));
}
