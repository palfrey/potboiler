use hyper;
use iron::prelude::{IronError, IronResult, Request, Response};
use iron::status;
use iron::typemap::Key;
use persistent;
use persistent::State;
use postgres;
use postgres::error::SqlState;
use potboiler_common::{db, url_from_body};
use potboiler_common::string_error::StringError;

use r2d2;
use r2d2_postgres;
use serde_json;
use serde_types::Log;
use std::collections::HashMap;
use std::io::Read;

use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use url::Url;
use uuid::Uuid;
pub type PostgresConnection = r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>;
pub type PostgresPool = r2d2::Pool<r2d2_postgres::PostgresConnectionManager>;
use clock;
use hybrid_clocks::{Timestamp, WallT, Wall, Clock};
use std::sync::RwLock;
pub type SyncClock = Arc<RwLock<Clock<Wall>>>;

#[derive(Copy, Clone)]
pub struct Nodes;

pub struct NodeInfo {
}

pub struct NodeList {
    nodes: HashMap<String, NodeInfo>,
    pool: PostgresPool,
    clock: SyncClock,
}

impl Key for Nodes {
    type Value = NodeList;
}

fn parse_object_from_request(raw_result: Result<hyper::client::Response, hyper::Error>)
                             -> Result<serde_json::value::Map<String, serde_json::Value>, StringError> {
    let mut res = match raw_result {
        Ok(val) => val,
        Err(val) => {
            return Err(StringError(format!("Failed to get logs: {:?}", val)));
        }
    };
    let body_string = {
        let mut body = String::new();
        res.read_to_string(&mut body).expect("could read from body");
        body
    };
    let json: serde_json::Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => {
            debug!("Good data back {:?}", val);
            val
        }
        Err(_) => return Err(StringError(format!("Got crappy data back: {:?}", body_string))),
    };
    return match json.as_object() {
        Some(val) => Ok(val.clone()),
        None => Err(StringError(format!("Info isn't a map: {:?}", json))),
    };
}

fn check_host_once(host_url: &String, conn: &PostgresConnection, clock_state: SyncClock) {
    let client = hyper::client::Client::new();
    let check_url = format!("{}/log", &host_url);
    info!("Checking {} ({})", host_url, check_url);
    let raw_result = client.get(&check_url).send();
    let kv = match parse_object_from_request(raw_result) {
        Ok(val) => val,
        Err(err) => {
            warn!("Error while getting from {:?}: {:?}", host_url, err);
            return;
        }
    };
    let stmt = conn.prepare("SELECT 1 from log where id=$1")
        .expect("prepare failure");
    for (key, value) in kv.iter() {
        let value_uuid = match Uuid::parse_str(value.as_str().unwrap()) {
            Ok(val) => val,
            Err(_) => {
                warn!("Value {} isn't a UUID!", value);
                continue;
            }
        };
        let key_uuid = match Uuid::parse_str(key) {
            Ok(val) => val,
            Err(_) => {
                warn!("Key {} isn't a UUID!", key);
                continue;
            }
        };
        let single_item = stmt.query(&[&value_uuid]).expect("bad single query");
        if !single_item.is_empty() {
            debug!("Already have {} locally", value_uuid);
            continue;
        }

        let first_item = conn.query("SELECT id from log WHERE prev is null and owner=$1 limit 1",
                   &[&key_uuid])
            .expect("bad first query");
        let start_uuid = if first_item.is_empty() {
            let first_url = format!("{}/log/first", &host_url);
            debug!("Get first from {:?}", host_url);
            let res = client.get(&first_url).send();
            let first_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => {
                    warn!("Error while getting first item from {:?}: {:?}",
                          host_url,
                          err);
                    return;
                }
            };
            info!("First entry: {:?}", first_entry);
            first_entry.get(key).unwrap().clone()
        } else {
            info!("Already have an entry from the list with server id {:?}",
                  key);
            let last_items = conn.query("SELECT id from log WHERE next is null and owner=$1 limit 1",
                       &[&key_uuid])
                .expect("bad last query");
            if last_items.is_empty() {
                warn!("Can't find end entry for server id {:?}", key);
                return;
            }
            let last_item = last_items.get(0);
            let last_item_id: Uuid = last_item.get("id");
            info!("Last item: {:?}", last_item_id);
            let last_url = format!("{}/log/{}", &host_url, last_item_id);
            debug!("Get last from {:?}", host_url);
            let res = client.get(&last_url).send();
            let last_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => {
                    warn!("Error while getting first item from {:?}: {:?}",
                          host_url,
                          err);
                    return;
                }
            };
            info!("Last entry: {:?}", last_entry);
            last_entry.get("next").unwrap().clone()
        };
        let mut current_uuid = start_uuid;
        loop {
            let real_uuid = {
                let str_uuid = current_uuid.as_str();
                String::from(str_uuid.expect("UUID unwrap"))
            };
            let current_url = format!("{}/log/{}", &host_url, real_uuid);
            debug!("Get {} from {}", real_uuid, host_url);
            let res = client.get(&current_url).send();
            let current_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => {
                    warn!("Error while getting first item from {:?}: {:?}",
                          host_url,
                          err);
                    return;
                }
            };
            let next = current_entry.get("next").expect("has next key");
            let timestamp: Timestamp<WallT> =
                serde_json::value::from_value(current_entry.get("when").expect(" has when key").clone())
                    .expect("is byte string");
            clock::observe_timestamp(&clock_state, timestamp);
            let log = Log {
                id: Uuid::parse_str(&real_uuid).unwrap(),
                owner: key_uuid,
                next: get_uuid_from_map(&current_entry, "next"),
                prev: get_uuid_from_map(&current_entry, "prev"),
                data: current_entry.get("data").unwrap().clone(),
                when: clock::get_timestamp_from_state(&clock_state),
            };
            insert_log(conn, &log);
            if next.is_null() {
                break;
            }
            current_uuid = next.clone();
        }
    }
}

fn get_uuid_from_map(map: &serde_json::value::Map<String, serde_json::Value>, key: &str) -> Option<Uuid> {
    let value = map.get(key).unwrap();
    if value.is_null() {
        None
    } else {
        Some(Uuid::parse_str(value.as_str().unwrap()).unwrap())
    }
}

pub fn insert_log(conn: &PostgresConnection, log: &Log) {
    debug!("Inserting {:?}", log);
    if log.prev.is_some() {
        conn.execute("UPDATE log set next = $1 where owner = $2 and id = $3",
                     &[&log.id, &log.owner, &log.prev])
            .expect("update worked");
    }
    let raw_timestamp = clock::get_raw_timestamp(&log.when);
    conn.execute("INSERT INTO log (id, owner, data, prev, hlc_tstamp) VALUES ($1, $2, $3, $4, $5)",
                 &[&log.id, &log.owner, &log.data, &log.prev, &raw_timestamp])
        .expect("insert worked");
}

fn check_host(host_url: String, conn: PostgresConnection, clock_state: SyncClock) {
    let sleep_time = Duration::from_secs(5);
    loop {
        check_host_once(&host_url, &conn, clock_state.clone());
        thread::sleep(sleep_time);
    }
}

pub fn initial_nodes(pool: PostgresPool, clock_state: SyncClock) -> NodeList {
    let conn = pool.get().unwrap();
    let mut nodes = HashMap::new();
    let stmt = conn.prepare("select url from nodes").expect("prepare failure");
    for row in &stmt.query(&[]).expect("nodes select works") {
        let url: String = row.get("url");
        nodes.insert(url.clone(), NodeInfo {});
        let conn = pool.get().unwrap();
        let cs = clock_state.clone();
        thread::spawn(move || check_host(url.clone(), conn, cs));
    }
    return NodeList {
        nodes: nodes,
        pool: pool,
        clock: clock_state,
    };
}

fn get_nodes_list(req: &Request) -> Vec<String> {
    let state_ref = req.extensions.get::<State<Nodes>>().unwrap().read().unwrap();
    let nodelist = state_ref.deref();
    let mut vec = Vec::with_capacity(nodelist.nodes.len());
    for key in nodelist.nodes.keys() {
        vec.push(key.clone());
    }
    return vec;
}

fn insert_node(req: &mut Request, to_notify: &String) {
    let (conn, clock_state) = {
        let mut nodelist = req.extensions
            .get_mut::<State<Nodes>>()
            .unwrap()
            .write()
            .unwrap();
        let nodelist_dm = nodelist.deref_mut();
        nodelist_dm.nodes.insert(to_notify.clone(), NodeInfo {});
        (nodelist_dm.pool.get().unwrap(), nodelist_dm.clock.clone())
    };

    let url = to_notify.clone();
    thread::spawn(move || check_host(url, conn, clock_state));
}

pub fn notify_everyone(req: &Request, log_arc: Arc<Log>) {
    let nodes = get_nodes_list(req);
    for node in nodes {
        let local_log = log_arc.clone();
        thread::spawn(move || {
            let client = hyper::client::Client::new();
            let notify_url = format!("{}/log/other", node);
            debug!("Notifying (node) {}", notify_url);
            let res = client.post(&notify_url)
                .body(&serde_json::ser::to_string(&local_log).unwrap())
                .send();
            match res {
                Ok(val) => {
                    if val.status != hyper::status::StatusCode::Ok {
                        warn!("Failed to notify {:?}: {:?}", &node, val.status);
                    }
                }
                Err(val) => {
                    warn!("Failed to notify {:?}: {:?}", &node, val);
                }
            };
        });
    }
}

pub fn node_add(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let url = url_from_body(req).unwrap().unwrap();
    debug!("Registering node {}", url);
    match Url::parse(&url) {
        Err(err) => Err(IronError::new(err, (status::BadRequest, "Bad URL"))),
        Ok(_) => {
            match conn.execute("INSERT INTO nodes (url) VALUES ($1)", &[&url]) {
                Ok(_) => {
                    insert_node(req, &url);
                    Ok(Response::with((status::NoContent)))
                }
                Err(err) => {
                    if let postgres::error::Error::Db(dberr) = err {
                        match dberr.code {
                            SqlState::UniqueViolation => Ok(Response::with((status::NoContent))),
                            _ => Err(IronError::new(dberr, (status::BadRequest, "Some other error"))),
                        }
                    } else {
                        Err(IronError::new(err, (status::BadRequest, "Some other error")))
                    }
                }
            }
        }
    }
}

pub fn node_remove(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    conn.execute("DELETE from nodes where url = $1",
                 &[&url_from_body(req).unwrap().unwrap()])
        .expect("delete worked");
    Ok(Response::with((status::NoContent)))
}

pub fn node_list(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let stmt = conn.prepare("select url from nodes").expect("prepare failure");
    let mut nodes = Vec::new();
    for row in &stmt.query(&[]).expect("last select works") {
        let url: String = row.get("url");
        nodes.push(url);
    }
    Ok(Response::with((status::Ok, serde_json::ser::to_string(&nodes).unwrap())))
}
