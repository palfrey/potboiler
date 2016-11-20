use clock;
use hybrid_clocks::{Clock, Timestamp, Wall, WallT};
use hyper;
use iron::prelude::{IronError, IronResult, Request, Response};
use iron::status;
use iron::typemap::Key;
use persistent;
use persistent::State;
use plugin::Pluggable;
use postgres;
use postgres::error::SqlState;
use potboiler_common::{db, url_from_body};
use potboiler_common::string_error::StringError;
use potboiler_common::types::Log;
use r2d2;
use r2d2_postgres;
use resolve;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::Read;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, RwLock};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;
use url::Url;
use urlencoded::UrlEncodedQuery;
use uuid::Uuid;

pub type LockedNode = Arc<RwLock<HashMap<String, NodeInfo>>>;
pub type PostgresConnection = r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>;
pub type PostgresPool = r2d2::Pool<r2d2_postgres::PostgresConnectionManager>;
pub type SyncClock = Arc<RwLock<Clock<Wall>>>;

#[derive(Copy, Clone)]
pub struct Nodes;

pub struct NodeInfo {
    sender: Mutex<Sender<()>>,
}

#[derive(Clone)]
pub struct NodeList {
    nodes: LockedNode,
    pool: PostgresPool,
    clock: SyncClock,
}

impl Key for Nodes {
    type Value = NodeList;
}

fn parse_json_from_request(raw_result: Result<hyper::client::Response, hyper::Error>)
                           -> Result<serde_json::value::Value, StringError> {
    let mut res = match raw_result {
        Ok(val) => val,
        Err(val) => {
            return Err(StringError(format!("Failed to get logs: {:?}", val)));
        }
    };
    let body_string = {
        let mut body = String::new();
        try!(res.read_to_string(&mut body));
        body
    };
    return match serde_json::de::from_str(&body_string) {
        Ok(val) => {
            debug!("Good data back {:?}", val);
            Ok(val)
        }
        Err(_) => Err(StringError(format!("Got crappy data back: {:?}", body_string))),
    };
}

fn parse_object_from_request(raw_result: Result<hyper::client::Response, hyper::Error>)
                             -> Result<serde_json::value::Map<String, serde_json::Value>, StringError> {
    let json: serde_json::Value = match parse_json_from_request(raw_result) {
        Ok(val) => val,
        Err(err) => return Err(err),
    };
    return match json.as_object() {
        Some(val) => Ok(val.clone()),
        None => Err(StringError(format!("Info isn't a map: {:?}", json))),
    };
}

fn check_host_once(host_url: &String,
                   conn: &PostgresConnection,
                   clock_state: SyncClock)
                   -> Result<(), StringError> {
    let mut client = hyper::client::Client::new();
    client.set_read_timeout(Some(Duration::from_secs(10)));
    client.set_write_timeout(Some(Duration::from_secs(10)));
    let check_url = format!("{}/log", &host_url);
    info!("Checking {} ({})", host_url, check_url);
    let raw_result = client.get(&check_url).send();
    let kv = match parse_object_from_request(raw_result) {
        Ok(val) => val,
        Err(err) => {
            return Err(StringError::from(format!("Error while getting from {:?}: {:?}", host_url, err)));
        }
    };
    let stmt = try!(conn.prepare("SELECT 1 from log where id=$1"));
    for (key, value) in kv.iter() {
        let value_uuid = match Uuid::parse_str(try!(value.as_str()
            .ok_or(StringError::from("Not a UUID!")))) {
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
        let single_item = try!(stmt.query(&[&value_uuid]));
        if !single_item.is_empty() {
            debug!("Already have {} locally", value_uuid);
            continue;
        }

        let first_item = try!(conn.query("SELECT id from log WHERE prev is null and owner=$1 limit 1",
                                         &[&key_uuid]));
        let start_uuid = if first_item.is_empty() {
            let first_url = format!("{}/log/first", &host_url);
            debug!("Get first from {:?}", host_url);
            let res = client.get(&first_url).send();
            let first_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => {
                    return Err(StringError::from(format!("Error while getting first item from {:?}: {:?}",
                                                         host_url,
                                                         err)));
                }
            };
            info!("First entry: {:?}", first_entry);
            try!(first_entry.get(key).ok_or(StringError::from(format!("Can't find {} key", key)))).clone()
        } else {
            info!("Already have an entry from the list with server id {:?}",
                  key);
            let last_items = try!(conn.query("SELECT id from log WHERE next is null and owner=$1 limit 1",
                                             &[&key_uuid]));
            if last_items.is_empty() {
                return Err(StringError::from(format!("Can't find end entry for server id {:?}", key)));
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
                    return Err(StringError::from(format!("Error while getting first item from {:?}: {:?}",
                                                         host_url,
                                                         err)));
                }
            };
            info!("Last entry: {:?}", last_entry);
            try!(last_entry.get("next").ok_or(StringError::from("No next key!"))).clone()
        };
        let mut current_uuid = start_uuid;
        loop {
            let real_uuid = {
                let str_uuid = current_uuid.as_str();
                String::from(try!(str_uuid.ok_or(format!("Current id ({}) is not a UUID", current_uuid))))
            };
            let current_url = format!("{}/log/{}", &host_url, real_uuid);
            debug!("Get {} from {}", real_uuid, host_url);
            let res = client.get(&current_url).send();
            let current_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => {
                    return Err(StringError::from(format!("Error while getting first item from {:?}: {:?}",
                                                         host_url,
                                                         err)));
                }
            };
            let next = try!(current_entry.get("next").ok_or(StringError::from("No next key!")));
            let timestamp: Timestamp<WallT> = try!(serde_json::value::from_value(
                    try!(current_entry.get("when").ok_or(StringError::from("No when key!")))
                    .clone()));
            clock::observe_timestamp(&clock_state, timestamp);
            let log = Log {
                id: try!(Uuid::parse_str(&real_uuid)),
                owner: key_uuid,
                next: get_uuid_from_map(&current_entry, "next"),
                prev: get_uuid_from_map(&current_entry, "prev"),
                data: try!(current_entry.get("data").ok_or(StringError::from("No data key"))).clone(),
                when: clock::get_timestamp_from_state(&clock_state),
            };
            insert_log(conn, &log);
            if next.is_null() {
                break;
            }
            current_uuid = next.clone();
        }
    }
    return Ok(());
}

fn get_uuid_from_map(map: &serde_json::value::Map<String, serde_json::Value>, key: &str) -> Option<Uuid> {
    let value = match map.get(key) {
        Some(val) => val,
        None => {
            return None;
        }
    };
    if value.is_null() {
        return None;
    }
    return match value.as_str() {
        Some(val) => {
            return Uuid::parse_str(val).ok();
        }
        None => None,
    };
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

fn hashset_from_json_array(nodes: &Vec<serde_json::Value>) -> Result<HashSet<String>, StringError> {
    let mut ret = HashSet::new();
    for node in nodes {
        match node.as_str() {
            None => {
                return Err(StringError::from("remote_node wasn't a string!"));
            }
            Some(val) => {
                ret.insert(String::from(val));
            }
        }
    }
    return Ok(ret);
}

fn check_new_nodes(host_url: &String,
                   conn: &PostgresConnection,
                   nodelist: NodeList)
                   -> Result<(), StringError> {
    let client = hyper::client::Client::new();
    let check_url = format!("{}/nodes?query_port=8000", &host_url);
    info!("Checking {} ({})", host_url, check_url);
    let raw_result = client.get(&check_url).send();
    let remote_nodes = match parse_json_from_request(raw_result) {
        Ok(val) => val,
        Err(err) => {
            warn!("Error while getting from {}: {:?}", host_url, err);
            return Err(err);
        }
    };
    let remote_node_array = try!(remote_nodes.as_array()
        .ok_or(StringError::from("remote_nodes isn't an array!")));
    let remote_node_set: HashSet<String> = try!(hashset_from_json_array(remote_node_array));
    let existing_nodes = try!(conn.query("SELECT url from nodes", &[]));
    let existing_nodes_set: HashSet<String> = HashSet::from_iter(existing_nodes.iter()
        .map(|x| x.get::<&str, String>("url")));
    let extra_nodes = remote_node_set.difference(&existing_nodes_set)
        .map(|x| x.clone())
        .collect::<Vec<String>>();
    debug!("From {} remote nodes: {:?}", check_url, remote_node_set);
    info!("Extra nodes from {}: {:?}", check_url, extra_nodes);
    let mut nodes = try!(nodelist.nodes.write().map_err(|x| StringError::from(x.description())));
    for extra in extra_nodes {
        match node_insert(&conn, &extra) {
            InsertResult::Inserted => {
                let (send, recv) = channel();
                nodes.insert(extra.clone(), NodeInfo { sender: Mutex::new(send) });
                let nodeslist = nodelist.clone();
                thread::spawn(move || check_host(extra.clone(), nodeslist, recv));
            }
            InsertResult::Existing => {}
            InsertResult::Error(err) => {
                warn!("Error while inserting node: {:?}", err);
            }
        }
    }
    return Ok(());
}

macro_rules! check_should_exit {
    ($recv:ident, $host_url:ident) => {{
        match $recv.try_recv() {
            Ok(_) => {
                // Only message is "quit" at the moment
                info!("Quitting check thread for {}", $host_url);
                return;
            }
            Err(_) => {}
        };
    }};
}

fn check_host(host_url: String, nodelist: NodeList, recv: Receiver<()>) {
    let sleep_time = Duration::from_secs(5);
    let conn = nodelist.pool.get().unwrap();
    loop {
        check_should_exit!(recv, host_url);
        match check_host_once(&host_url, &conn, nodelist.clock.clone()) {
            Ok(_) => {}
            Err(msg) => {
                warn!("Got an error while checking for new log items on {}: {}",
                      host_url,
                      msg);
            }
        };
        check_should_exit!(recv, host_url);
        match check_new_nodes(&host_url, &conn, nodelist.clone()) {
            Ok(_) => {}
            Err(msg) => {
                warn!("Got an error while checking for new nodes on {}: {}",
                      host_url,
                      msg);
            }
        }
        check_should_exit!(recv, host_url);
        thread::sleep(sleep_time);
    }
}

pub fn initial_nodes(pool: PostgresPool, clock_state: SyncClock) -> NodeList {
    let conn = pool.get().unwrap();
    let stmt = conn.prepare("select url from nodes").expect("prepare failure");
    let locked_nodes = Arc::new(RwLock::new(HashMap::new()));
    let mut nodes = locked_nodes.write().unwrap();
    for row in &stmt.query(&[]).expect("nodes select works") {
        let url: String = row.get("url");
        let (send, recv) = channel();
        nodes.insert(url.clone(), NodeInfo { sender: Mutex::new(send) });
        let nodeslist = NodeList {
            nodes: locked_nodes.clone(),
            pool: pool.clone(),
            clock: clock_state.clone(),
        };
        thread::spawn(move || check_host(url.clone(), nodeslist, recv));
    }
    return NodeList {
        nodes: locked_nodes.clone(),
        pool: pool,
        clock: clock_state,
    };
}

fn get_nodes_list(req: &Request) -> Vec<String> {
    let state_ref = req.extensions.get::<State<Nodes>>().unwrap().read().unwrap();
    let nodes = state_ref.deref().nodes.read().unwrap();
    let mut vec = Vec::with_capacity(nodes.len());
    for key in nodes.keys() {
        vec.push(key.clone());
    }
    return vec;
}

fn insert_node(req: &mut Request, to_notify: &String) {
    let (send, recv) = channel();
    let nodelist = {
        let mut nodelist = req.extensions
            .get_mut::<State<Nodes>>()
            .unwrap()
            .write()
            .unwrap();
        let nodelist_dm = nodelist.deref_mut();
        nodelist_dm.nodes.write().unwrap().insert(to_notify.clone(), NodeInfo { sender: Mutex::new(send) });
        NodeList {
            nodes: nodelist_dm.nodes.clone(),
            pool: nodelist_dm.pool.clone(),
            clock: nodelist_dm.clock.clone(),
        }
    };

    let url = to_notify.clone();
    thread::spawn(move || check_host(url, nodelist, recv));
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

enum InsertResult {
    Inserted,
    Existing,
    Error(postgres::error::Error),
}

fn node_insert(conn: &PostgresConnection, url: &String) -> InsertResult {
    return match conn.execute("INSERT INTO nodes (url) VALUES ($1)", &[&url]) {
        Ok(_) => InsertResult::Inserted,
        Err(err) => {
            if let postgres::error::Error::Db(dberr) = err {
                match dberr.code {
                    SqlState::UniqueViolation => InsertResult::Existing,
                    _ => InsertResult::Error(postgres::error::Error::Db(dberr)),
                }
            } else {
                InsertResult::Error(err)
            }
        }
    };
}

fn node_add_core(conn: &PostgresConnection,
                 url: &String,
                 req: &mut Request)
                 -> Result<(), postgres::error::Error> {
    match node_insert(&conn, &url) {
        InsertResult::Inserted => {
            insert_node(req, &url);
            Ok(())
        }
        InsertResult::Existing => Ok(()),
        InsertResult::Error(err) => Err(err),
    }
}

pub fn node_add(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let url = url_from_body(req).unwrap().unwrap();
    debug!("Registering node {}", url);
    match Url::parse(&url) {
        Err(err) => Err(IronError::new(err, (status::BadRequest, "Bad URL"))),
        Ok(_) => {
            match node_add_core(&conn, &url, req) {
                Ok(_) => Ok(Response::with((status::NoContent))),
                Err(err) => Err(IronError::new(err, (status::BadRequest, "Some other error"))),
            }
        }
    }
}

pub fn node_remove(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let notifier = url_from_body(req).unwrap().unwrap();
    conn.execute("DELETE from nodes where url = $1", &[&notifier])
        .expect("delete worked");
    let mut nodelist = req.extensions
        .get_mut::<State<Nodes>>()
        .unwrap()
        .write()
        .unwrap();
    let nodelist_dm = nodelist.deref_mut();
    let mut nodes = nodelist_dm.nodes.write().unwrap();
    {
        // Limit the immutable borrow of nodes
        let info = match nodes.get(&notifier) {
            Some(val) => val,
            None => {
                return Err(IronError::new(StringError::from(format!("No such notifier {} registered",
                                                                    &notifier)),
                                          (status::NotFound)));
            }
        };
        info.sender.lock().unwrap().deref().send(()).unwrap();
    }
    nodes.remove(&notifier);
    Ok(Response::with((status::NoContent)))
}

fn add_node_from_req(req: &mut Request,
                     nodes: &Vec<String>,
                     conn: &PostgresConnection)
                     -> Result<(), StringError> {
    let host = try!(resolve::resolver::resolve_addr(&req.remote_addr.ip()));
    let port = {
        let query = req.get_ref::<UrlEncodedQuery>();
        let values = try!(query.map_err(|x| StringError::from(x.description())));
        let ports = try!(values.get("query_port").ok_or(StringError::from("Can't find query_port")));
        let port_str = try!(ports.first().ok_or(StringError::from("Zero query_port's somehow...")));
        try!(port_str.parse::<u32>().map_err(|x| StringError::from(x.description())))
    };
    let query_url = format!("http://{}:{}", host, port);
    if !nodes.contains(&query_url) {
        info!("{} is missing from nodes", query_url);
        return node_add_core(conn, &query_url, req).map_err(|x| StringError::from(x.description()));
    }
    return Ok(());
}

pub fn node_list(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let stmt = conn.prepare("select url from nodes").expect("prepare failure");
    let mut nodes = Vec::new();
    for row in &stmt.query(&[]).expect("last select works") {
        let url: String = row.get("url");
        nodes.push(url);
    }
    if let Err(err) = add_node_from_req(req, &nodes, &conn) {
        warn!("Error from add_node_from_req: {}", err);
    }
    Ok(Response::with((status::Ok, serde_json::ser::to_string(&nodes).unwrap())))
}
