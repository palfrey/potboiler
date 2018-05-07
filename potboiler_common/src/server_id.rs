use iron::typemap::Key;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

#[derive(Copy, Clone)]
pub struct ServerId;

impl Key for ServerId {
    type Value = Uuid;
}

#[macro_export]
macro_rules! get_server_id {
    ($req:expr) => (match $req.extensions.get::<persistent::Read<server_id::ServerId>>() {
        Some(id) => id,
        None => {
            println!("Couldn't get the server id from the request!");
            return Ok(Response::with(status::InternalServerError));
        }
    })
}

pub fn setup() -> Uuid {
    let id_path = &env::var("ID_PATH").unwrap_or("server-id".to_string());
    if !Path::new(id_path).exists() {
        let mut f = File::create(id_path).expect(&format!("Can create {}", id_path));
        let id = Uuid::new_v4();
        f.write_fmt(format_args!("{}", id.hyphenated())).expect(&format!("Can write {}", id_path));
        id
    } else {
        let mut f = File::open(id_path).expect(&format!("Can open {}", id_path));
        let mut s = String::new();
        f.read_to_string(&mut s).expect(&format!("Can read {}", id_path));
        Uuid::parse_str(&s).expect(&format!("Can parse '{}' as UUID", s))
    }
}

pub fn test() -> Uuid {
    Uuid::parse_str("FEEDFACEDEADFEEDFACEDEADFACEDEAD").unwrap()
}