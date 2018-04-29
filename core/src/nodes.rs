use hybrid_clocks::{Clock, Timestamp, Wall, WallT};
use hyper;
use iron::prelude::{IronError, IronResult, Request, Response};
use iron::status;
use iron::typemap::Key;
use persistent;
use persistent::State;
use plugin::Pluggable;
use potboiler_common::{clock, db, get_raw_timestamp, url_from_body};
use potboiler_common::types::Log;
use resolve;
use serde_json;
use std::result::Result as StdResult;
use std::convert;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, RwLock};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;
use url::Url;
use urlencoded::{UrlDecodingError, UrlEncodedQuery};
use uuid::Uuid;
use schema::logs as log_table;
use diesel;
use std::marker;
use diesel::QueryDsl;

pub type LockedNode = Arc<RwLock<HashMap<String, NodeInfo>>>;
pub type SyncClock = Arc<RwLock<Clock<Wall>>>;

#[derive(Copy, Clone)]
pub struct Nodes;

pub struct NodeInfo {
    sender: Mutex<Sender<()>>,
}

#[derive(Clone)]
pub struct NodeList {
    nodes: LockedNode,
    pool: db::Pool,
    clock: SyncClock,
}

impl Key for Nodes {
    type Value = NodeList;
}

#[derive(Debug, Queryable)]
pub struct NodeTable {
    pub url: String
}

error_chain! {
    errors {
        HostRetrieveError(host_url: String)
        NoQueryPort
        PoisonError
        NonArrayRemoteNodes(nodes: serde_json::Value)
        NoSuchNotifier(notifier: String)
        NonStringRemoteNode(node: String)
        CrappyData(node: String)
        NotAMap(node: serde_json::Value)
        NoDataKey
        NoWhenKey
        NoNextKey
        KeyMissing(key: String)
        BadLogUuid(uuid: String)
        BadUuid(uuid: String)
        NoLastEntry(key: String)
    }
    links {
        DbError(db::Error, db::ErrorKind);
    }
    foreign_links {
        ParseIntError(::std::num::ParseIntError);
        Io(::std::io::Error);
        UrlEncoding(UrlDecodingError);
        LogFailure(hyper::Error);
        SerdeError(serde_json::Error);
    }
}

fn parse_json_from_request(raw_result: StdResult<hyper::client::Response, hyper::Error>)
                           -> Result<serde_json::value::Value> {
    let mut res = match raw_result {
        Ok(val) => val,
        Err(val) => {
            bail!(ErrorKind::LogFailure(val));
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
        Err(_) => bail!(ErrorKind::CrappyData(body_string))
    };
}

fn parse_object_from_request(raw_result: StdResult<hyper::client::Response, hyper::Error>)
                             -> Result<serde_json::value::Map<String, serde_json::Value>> {
    let json: serde_json::Value = match parse_json_from_request(raw_result) {
        Ok(val) => val,
        Err(err) => return Err(err),
    };
    return match json.as_object() {
        Some(val) => Ok(val.clone()),
        None => bail!(ErrorKind::NotAMap(json))
    };
}

fn check_host_once(host_url: &String,
                   conn: &C,
                   clock_state: SyncClock)
                   -> Result<()> {
    let mut client = hyper::client::Client::new();
    client.set_read_timeout(Some(Duration::from_secs(10)));
    client.set_write_timeout(Some(Duration::from_secs(10)));
    let check_url = format!("{}/log", &host_url);
    info!("Checking {} ({})", host_url, check_url);
    let raw_result = client.get(&check_url).send();
    let kv = match parse_object_from_request(raw_result) {
        Ok(val) => val,
        Err(err) => {
            bail!(Error::with_chain(err, ErrorKind::HostRetrieveError(host_url.clone())));
        }
    };
    for (key, value) in kv.iter() {
        let value_uuid = match Uuid::parse_str(value.as_str()
            .ok_or(ErrorKind::BadUuid(serde_json::to_string(value)?))?) {
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
        let single_item = log_table::table.filter(log_table::id.eq(value_uuid));
        if !single_item.is_empty() {
            debug!("Already have {} locally", value_uuid);
            continue;
        }

        let first_item = log_table::table()
            .filter(log_table::prev().is_null())
            .filter(log_table::owner().eq(key_uuid))
            .limit(1)
            .get_result::<Log>(&conn)?;
        let start_uuid = if first_item.is_empty() {
            let first_url = format!("{}/log/first", &host_url);
            debug!("Get first from {:?}", host_url);
            let res = client.get(&first_url).send();
            let first_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => {
                    bail!(Error::with_chain(err, ErrorKind::HostRetrieveError(host_url.clone())));
                }
            };
            info!("First entry: {:?}", first_entry);
            first_entry.get(key).ok_or(ErrorKind::KeyMissing(key.clone()))?
        } else {
            info!("Already have an entry from the list with server id {:?}",
                  key);
            let last_items = log_table::table
                .filter(log_table::next.is_null())
                .filter(log_table::owner.eq(key_uuid))
                .limit(1)
                .get_result::<Log>(&conn)?;
            if last_items.is_empty() {
                bail!(ErrorKind::NoLastEntry(key.clone()));
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
                    bail!(Error::with_chain(err, ErrorKind::HostRetrieveError(host_url.clone())));
                }
            };
            info!("Last entry: {:?}", last_entry);
            last_entry.get("next").ok_or(ErrorKind::NoNextKey)?
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
                    bail!(Error::with_chain(err, ErrorKind::HostRetrieveError(host_url.clone())));
                }
            };
            let next = current_entry.get("next").ok_or(ErrorKind::NoNextKey)?;
            let timestamp: Timestamp<WallT> = serde_json::value::from_value(
                    current_entry.get("when").ok_or(ErrorKind::NoWhenKey)?
                    .clone())?;
            clock::observe_timestamp(&clock_state, timestamp);
            let log = Log {
                id: Uuid::parse_str(&real_uuid).map_err(|_| ErrorKind::BadLogUuid(real_uuid))?,
                owner: key_uuid,
                next: get_uuid_from_map(&current_entry, "next"),
                prev: get_uuid_from_map(&current_entry, "prev"),
                data: try!(current_entry.get("data").ok_or(ErrorKind::NoDataKey)).clone(),
                when: clock::get_timestamp_from_state(&clock_state),
            };
            insert_log(conn, &log);
            if next.is_null() {
                break;
            }
            current_uuid = &next;
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

#[derive(Debug, Insertable)]
#[table_name="log_table"]
pub struct NewLog {
    pub id: Uuid,
    pub owner: Uuid,
    pub prev: Option<Uuid>,
    pub next: Option<Uuid>,
    pub hlc_tstamp: Vec<u8>,
    pub data: serde_json::Value,
}

pub fn insert_log(conn: &C, log: &Log) -> Result<()> {
    debug!("Inserting {:?}", log);
    if let Some(prev) = log.prev {
        diesel::update(log_table::table.filter(log_table::owner.eq(log.owner)).filter(log_table::id.eq(prev)))
            .set(log_table::next.eq(log.id))
            .get_result::<Log>(conn)
            .expect("update worked");
    }
    let raw_timestamp = get_raw_timestamp(&log.when);

    let new_log = NewLog {
        id: log.id,
        owner: log.owner,
        prev: log.prev,
        next: None,
        when: raw_timestamp,
        data: log.data
    };
    
    diesel::insert_into(log_table::table)
        .values(new_log)
        .get_result(conn)?;
    Ok(())
}

fn hashset_from_json_array(nodes: &Vec<serde_json::Value>) -> Result<HashSet<String>> {
    let mut ret = HashSet::new();
    for node in nodes {
        match node.as_str() {
            None => {
                bail!(ErrorKind::NonStringRemoteNode(serde_json::to_string(node).unwrap()));
            }
            Some(val) => {
                ret.insert(String::from(val));
            }
        }
    }
    return Ok(ret);
}

fn check_new_nodes(host_url: &String,
                   conn: &C,
                   nodelist: NodeList)
                   -> Result<()> {
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
    let remote_node_array = remote_nodes.as_array()
        .ok_or(ErrorKind::NonArrayRemoteNodes(remote_nodes))?;
    let remote_node_set: HashSet<String> = try!(hashset_from_json_array(remote_node_array));
    let existing_nodes = conn.query("SELECT url from nodes")?;
    let existing_nodes_set: HashSet<String> = HashSet::from_iter(existing_nodes.iter()
        .map(|x| x.get("url")));
    let extra_nodes = remote_node_set.difference(&existing_nodes_set)
        .map(|x| x.clone())
        .collect::<Vec<String>>();
    debug!("From {} remote nodes: {:?}", check_url, remote_node_set);
    info!("Extra nodes from {}: {:?}", check_url, extra_nodes);
    let mut nodes = nodelist.nodes.write().map_err(|_| ErrorKind::PoisonError)?;
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
    let conn = nodelist.pool.get().unwrap().connect().unwrap();
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

pub fn initial_nodes(pool: db::Pool, clock_state: SyncClock) -> Result<NodeList>
    {
    let conn = pool.get()?.connect()?;
    let locked_nodes = Arc::new(RwLock::new(HashMap::new()));
    let mut nodes = locked_nodes.write().unwrap();
    for row in &conn.query("select url from nodes").expect("nodes select works") {
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
    return Ok(NodeList {
        nodes: locked_nodes.clone(),
        pool: pool,
        clock: clock_state,
    });
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
                .body(&serde_json::ser::to_string(&local_log.deref()).unwrap())
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
    Error(Error),
}

fn node_insert(conn: &C, url: &String) -> InsertResult 
    {
    let query = &NodeTable::table().insert_fields(&[&NodeTable::url()]);
    query.push_untyped(&[url.as_expr()]);
    return match conn.dexecute(query) {
        Ok(_) => InsertResult::Inserted,
        Err(db::Error(db::ErrorKind::UniqueViolation, _)) => {
            InsertResult::Existing
        },
        Err(err) => {
            InsertResult::Error(convert::From::from(err))
        }
    };
}

fn node_add_core(conn: &C,
                 url: &String,
                 req: &mut Request)
                 -> Result<()>
                 {
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
    let conn = get_db_connection!(&req);
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
    let conn = get_db_connection!(&req);
    let notifier = url_from_body(req).unwrap().unwrap();
    conn.dexecute(&NodeTable::table().delete().where_(NodeTable::url().is(&notifier)))
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
                return Err(IronError::new(Error::from_kind(ErrorKind::NoSuchNotifier(notifier)),
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
                     conn: &C)
                     -> Result<()> {
    let host = try!(resolve::resolver::resolve_addr(&req.remote_addr.ip()));
    let port = {
        let query = req.get_ref::<UrlEncodedQuery>()?;
        let ports = query.get("query_port").ok_or(ErrorKind::NoQueryPort)?;
        let port_str = ports.first().ok_or(ErrorKind::NoQueryPort)?;
        port_str.parse::<u32>()?
    };
    let query_url = format!("http://{}:{}", host, port);
    if !nodes.contains(&query_url) {
        info!("{} is missing from nodes", query_url);
        return node_add_core(conn, &query_url, req);
    }
    return Ok(());
}

pub fn node_list(req: &mut Request) -> IronResult<Response> {
    let conn = get_db_connection!(&req);
    let mut nodes = Vec::new();
    for row in conn.query("select url from nodes").expect("last select works").iter() {
        let url: String = row.get("url");
        nodes.push(url);
    }
    if let Err(err) = add_node_from_req(req, &nodes, &conn) {
        warn!("Error from add_node_from_req: {}", err);
    }
    Ok(Response::with((status::Ok, serde_json::ser::to_string(&nodes).unwrap())))
}
