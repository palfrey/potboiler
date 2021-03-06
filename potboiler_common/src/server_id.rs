use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
};
use uuid::Uuid;

pub fn setup() -> Uuid {
    let id_path = &env::var("ID_PATH").unwrap_or_else(|_| "server-id".to_string());
    if !Path::new(id_path).exists() {
        let mut f = File::create(id_path).unwrap_or_else(|_| panic!("Can create {}", id_path));
        let id = Uuid::new_v4();
        f.write_fmt(format_args!("{}", id.hyphenated()))
            .unwrap_or_else(|_| panic!("Can write {}", id_path));
        id
    } else {
        let mut f = File::open(id_path).unwrap_or_else(|_| panic!("Can open {}", id_path));
        let mut s = String::new();
        f.read_to_string(&mut s)
            .unwrap_or_else(|_| panic!("Can read {}", id_path));
        Uuid::parse_str(&s).unwrap_or_else(|_| panic!("Can parse '{}' as UUID", s))
    }
}

pub fn test() -> Uuid {
    Uuid::parse_str("feedface-dead-feed-face-deadfacedead").unwrap()
}
