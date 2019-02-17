use crate::clock::SyncClock;
use crate::AppState;
use actix_web::{HttpRequest, HttpResponse, Json, Query, State};
use failure::{bail, format_err, Error, Fail};
use hybrid_clocks::{Timestamp, WallT};
use log::{debug, info, warn};
use potboiler_common::{db, get_raw_timestamp, types::Log};
use resolve;
use serde_derive::Deserialize;
use serde_json;
use std::borrow::Borrow;
use std::{
    collections::{HashMap, HashSet},
    convert,
    iter::FromIterator,
    ops::Deref,
    result::Result as StdResult,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
    thread,
    time::Duration,
};
use url::Url;
use uuid::Uuid;

pub type LockedNode = Arc<RwLock<HashMap<String, NodeInfo>>>;

#[derive(Debug)]
pub struct NodeInfo {
    sender: Mutex<Sender<()>>,
}

#[derive(Clone, Debug)]
pub struct NodeList {
    nodes: LockedNode,
    pool: db::Pool,
    clock: SyncClock,
}

#[derive(Debug, Fail)]
enum NodesError {
    #[fail(display = "host retrieve error: {}", host_url)]
    HostRetrieveError {
        host_url: String,
        #[cause]
        cause: Error,
    },
    #[fail(display = "PoisonError")]
    PoisonError,
    #[fail(display = "NotAMap")]
    NotAMap { node: String },
    #[fail(display = "NoDataKey")]
    NoDataKey,
    #[fail(display = "NoWhenKey")]
    NoWhenKey,
    #[fail(display = "NoNextKey")]
    NoNextKey,
    #[fail(display = "KeyMissing")]
    KeyMissing { key: String },
    #[fail(display = "BadLogUuid")]
    BadLogUuid { uuid: String },
    #[fail(display = "BadUuid")]
    BadUuid { uuid: String },
    #[fail(display = "NoLastEntry")]
    NoLastEntry { key: String },
    #[fail(display = "NoSuchNotifier")]
    NoSuchNotifier { name: String },
}

fn parse_object_from_request(
    raw_result: StdResult<reqwest::Response, reqwest::Error>,
) -> Result<serde_json::value::Map<String, serde_json::Value>, Error> {
    let json: serde_json::Value = match raw_result?.json() {
        Ok(val) => val,
        Err(err) => bail!(err),
    };
    match json.as_object() {
        Some(val) => Ok(val.clone()),
        None => bail!(NodesError::NotAMap {
            node: serde_json::to_string(&json).unwrap()
        }),
    }
}

fn check_host_once(host_url: &str, conn: &db::Connection, clock_state: &SyncClock) -> Result<(), Error> {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(10)).build()?;
    let check_url = format!("{}/log", &host_url);
    info!("Checking {} ({})", host_url, check_url);
    let raw_result = client.get(&check_url).send();
    let kv = match parse_object_from_request(raw_result) {
        Ok(val) => val,
        Err(err) => bail!(NodesError::HostRetrieveError {
            host_url: host_url.to_string(),
            cause: err
        }),
    };
    for (key, value) in kv.iter() {
        let value_uuid = match Uuid::parse_str(value.as_str().ok_or_else(|| NodesError::BadUuid {
            uuid: serde_json::to_string(value).unwrap(),
        })?) {
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
        let single_item = conn.query(&format!("select 1 from log where id = '{}'", &value_uuid))?;
        if !single_item.is_empty() {
            debug!("Already have {} locally", value_uuid);
            continue;
        }

        let first_item = conn.query(&format!(
            "select id from log where prev is null and owner = '{}' limit 1",
            &key_uuid
        ))?;
        let start_uuid = if first_item.is_empty() {
            let first_url = format!("{}/log/first", host_url);
            debug!("Get first from {:?}", host_url);
            let res = client.get(&first_url).send();
            let first_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => bail!(NodesError::HostRetrieveError {
                    host_url: host_url.to_string(),
                    cause: err
                }),
            };
            info!("First entry: {:?}", &first_entry);
            first_entry
                .get(key)
                .ok_or_else(|| NodesError::KeyMissing { key: key.clone() })?
                .clone()
        } else {
            info!("Already have an entry from the list with server id {:?}", key);
            let last_items = conn.query(&format!(
                "select id from log where next is null and owner = '{}' limit 1",
                &key_uuid
            ))?;
            if last_items.is_empty() {
                bail!(NodesError::NoLastEntry { key: key.clone() });
            }
            let last_item = last_items.get(0);
            let last_item_id: Uuid = last_item.get("id");
            info!("Last item: {:?}", last_item_id);
            let last_url = format!("{}/log/{}", &host_url, last_item_id);
            debug!("Get last from {:?}", host_url);
            let res = client.get(&last_url).send();
            let last_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => bail!(NodesError::HostRetrieveError {
                    host_url: host_url.to_string(),
                    cause: err
                }),
            };
            info!("Last entry: {:?}", last_entry);
            last_entry.get("next").ok_or(NodesError::NoNextKey)?.clone()
        };
        let mut current_uuid = start_uuid;
        loop {
            let real_uuid = {
                let str_uuid = current_uuid.as_str();
                String::from(str_uuid.ok_or_else(|| format_err!("Current id ({}) is not a UUID", current_uuid))?)
            };
            let current_url = format!("{}/log/{}", host_url, real_uuid);
            debug!("Get {} from {}", real_uuid, host_url);
            let res = client.get(&current_url).send();
            let current_entry = match parse_object_from_request(res) {
                Ok(val) => val,
                Err(err) => bail!(NodesError::HostRetrieveError {
                    host_url: host_url.to_string(),
                    cause: err
                }),
            };
            let next = current_entry.get("next").ok_or(NodesError::NoNextKey)?;
            let timestamp: Timestamp<WallT> =
                serde_json::value::from_value(current_entry.get("when").ok_or(NodesError::NoWhenKey)?.clone())?;
            clock_state.observe_timestamp(timestamp);
            let log = Log {
                id: Uuid::parse_str(&real_uuid).map_err(|_| NodesError::BadLogUuid { uuid: real_uuid })?,
                owner: key_uuid,
                next: get_uuid_from_map(&current_entry, "next"),
                prev: get_uuid_from_map(&current_entry, "prev"),
                data: current_entry.get("data").ok_or(NodesError::NoDataKey)?.clone(),
                when: clock_state.get_timestamp(),
            };
            insert_log(conn, &log)?;
            if next.is_null() {
                break;
            }
            current_uuid = next.clone();
        }
    }
    Ok(())
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
    match value.as_str() {
        Some(val) => Uuid::parse_str(val).ok(),
        None => None,
    }
}

pub fn insert_log(conn: &db::Connection, log: &Log) -> Result<(), Error> {
    debug!("Inserting {:?}", log);
    if let Some(prev) = log.prev {
        conn.execute(&format!(
            "update log set next = {} where owner = '{}' and id = '{}'",
            &log.id, &log.owner, &prev
        ))?;
    }
    let raw_timestamp = get_raw_timestamp(&log.when)?;
    conn.execute(&format!(
        "insert into log (id, owner, data, prev, hlc_tstamp) VALUES ('{}', '{}', '{}', {}, {})",
        &log.id,
        &log.owner,
        &log.data,
        log.prev
            .map(|u| format!("'{}'", u.to_string()))
            .unwrap_or_else(|| String::from("NULL")),
        &raw_timestamp.sql()
    ))?;
    Ok(())
}

fn check_new_nodes(host_url: &str, conn: &db::Connection, nodelist: &NodeList) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let check_url = format!("{}/nodes?query_port=8000", host_url);
    info!("Checking {} ({})", host_url, check_url);
    let raw_result = client.get(&check_url).send();
    let remote_nodes: Vec<String> = match raw_result?.json() {
        Ok(val) => val,
        Err(err) => {
            warn!("Error while getting from {}: {:?}", host_url, err);
            bail!(err);
        }
    };
    let remote_node_set: HashSet<String> = HashSet::from_iter(remote_nodes);
    let existing_nodes = conn.query("select url from nodes")?;
    let existing_nodes_set: HashSet<String> = HashSet::from_iter(existing_nodes.iter().map(|x| x.get("url")));
    let extra_nodes = remote_node_set.difference(&existing_nodes_set).cloned();
    debug!("From {} remote nodes: {:?}", check_url, remote_node_set);
    info!("Extra nodes from {}: {:?}", check_url, extra_nodes);
    let mut nodes = nodelist.nodes.write().map_err(|_| NodesError::PoisonError)?;
    for extra in extra_nodes {
        match node_insert(&conn, &extra) {
            InsertResult::Inserted => {
                let (send, recv) = channel();
                nodes.insert(
                    extra.clone(),
                    NodeInfo {
                        sender: Mutex::new(send),
                    },
                );
                let nodeslist = nodelist.clone();
                thread::spawn(move || check_host(&extra, &nodeslist, &recv));
            }
            InsertResult::Existing => {}
            InsertResult::Error(err) => {
                warn!("Error while inserting node: {:?}", err);
            }
        }
    }
    Ok(())
}

macro_rules! check_should_exit {
    ($recv:ident, $host_url:ident) => {{
        if $recv.try_recv().is_ok() {
            // Only message is "quit" at the moment
            info!("Quitting check thread for {}", $host_url);
            return;
        };
    }};
}

fn check_host(host_url: &str, nodelist: &NodeList, recv: &Receiver<()>) {
    let sleep_time = Duration::from_secs(5);
    let conn = nodelist.pool.get().unwrap();
    loop {
        check_should_exit!(recv, host_url);
        match check_host_once(host_url, &conn, &nodelist.clock) {
            Ok(_) => {}
            Err(msg) => {
                warn!("Got an error while checking for new log items on {}: {}", host_url, msg);
            }
        };
        check_should_exit!(recv, host_url);
        match check_new_nodes(host_url, &conn, nodelist) {
            Ok(_) => {}
            Err(msg) => {
                warn!("Got an error while checking for new nodes on {}: {}", host_url, msg);
            }
        }
        check_should_exit!(recv, host_url);
        thread::sleep(sleep_time);
    }
}

pub fn initial_nodes(pool: db::Pool, clock_state: SyncClock) -> Result<NodeList, Error> {
    let conn = pool.get()?;
    let locked_nodes = Arc::new(RwLock::new(HashMap::new()));
    let mut nodes = locked_nodes.write().unwrap();
    for row in &conn.query("select url from nodes").expect("nodes select works") {
        let url: String = row.get("url");
        let (send, recv) = channel();
        nodes.insert(
            url.clone(),
            NodeInfo {
                sender: Mutex::new(send),
            },
        );
        let nodeslist = NodeList {
            nodes: locked_nodes.clone(),
            pool: pool.clone(),
            clock: clock_state.clone(),
        };
        thread::spawn(move || check_host(&url, &nodeslist, &recv));
    }
    Ok(NodeList {
        nodes: locked_nodes.clone(),
        pool,
        clock: clock_state,
    })
}

fn get_nodes_list(state: State<AppState>) -> Vec<String> {
    let nodes = state.nodes.nodes.read().unwrap();
    let mut vec = Vec::with_capacity(nodes.len());
    for key in nodes.keys() {
        vec.push(key.clone());
    }
    vec
}

fn insert_node(state: &AppState, to_notify: &str) {
    let (send, recv) = channel();
    let nodelist = {
        let mut nodelist = state.nodes.nodes.write().unwrap();
        nodelist.insert(
            to_notify.to_string(),
            NodeInfo {
                sender: Mutex::new(send),
            },
        );
        NodeList {
            nodes: state.nodes.nodes.clone(),
            pool: state.nodes.pool.clone(),
            clock: state.nodes.clock.clone(),
        }
    };
    let url = to_notify.to_string();
    thread::spawn(move || check_host(&url, &nodelist, &recv));
}

pub fn notify_everyone(state: State<AppState>, log_arc: &Arc<Log>) {
    let nodes = get_nodes_list(state);
    for node in nodes {
        let local_log = log_arc.clone();
        thread::spawn(move || {
            let client = reqwest::Client::new();
            let notify_url = format!("{}/log/other", node);
            debug!("Notifying (node) {}", notify_url);
            let res = client.post(&notify_url).json(&local_log.deref()).send();
            match res {
                Ok(val) => {
                    if val.status() != reqwest::StatusCode::OK {
                        warn!("Failed to notify {:?}: {:?}", &node, val.status());
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

fn node_insert(conn: &db::Connection, url: &str) -> InsertResult {
    match conn.execute(&format!("insert into nodes (url) values('{}')", url)) {
        Ok(_) => InsertResult::Inserted,
        Err(db::Error::UniqueViolation) => InsertResult::Existing,
        Err(err) => InsertResult::Error(convert::From::from(err)),
    }
}

fn node_add_core(conn: &db::Connection, url: &str, state: &AppState) -> Result<(), Error> {
    match node_insert(&conn, url) {
        InsertResult::Inserted => {
            insert_node(state, url);
            Ok(())
        }
        InsertResult::Existing => Ok(()),
        InsertResult::Error(err) => Err(err),
    }
}

#[derive(Deserialize)]
pub struct UrlJson {
    url: String,
}

pub fn node_add(state: State<AppState>, body: Json<UrlJson>) -> HttpResponse {
    let conn = state.pool.get().unwrap();
    let url = &body.url;
    debug!("Registering node {}", url);
    match Url::parse(&url) {
        Err(_) => HttpResponse::BadRequest().reason("Bad URL").finish(),
        Ok(_) => match node_add_core(&conn, &url, state.borrow()) {
            Ok(_) => HttpResponse::Created().finish(),
            Err(_err) => HttpResponse::BadRequest().reason("Some other error").finish(),
        },
    }
}

pub fn node_remove(state: State<AppState>, body: Json<UrlJson>) -> HttpResponse {
    let notifier = &body.url;
    state
        .pool
        .get()
        .unwrap()
        .execute(&format!("delete from nodes where url = '{}'", notifier))
        .expect("delete worked");
    let mut nodes = state.nodes.nodes.write().unwrap();
    let info = match nodes.get(notifier) {
        Some(val) => val,
        None => {
            return HttpResponse::NotFound().body(
                NodesError::NoSuchNotifier {
                    name: notifier.to_string(),
                }
                .to_string(),
            );
        }
    };
    info.sender.lock().unwrap().deref().send(()).unwrap();
    nodes.remove(notifier);
    HttpResponse::NoContent().finish()
}

fn add_node_from_req(
    query: Query<NodeListQuery>,
    req: HttpRequest<AppState>,
    nodes: &[String],
    conn: &db::Connection,
) -> Result<(), Error> {
    let host = resolve::resolver::resolve_addr(&req.connection_info().remote().unwrap().parse()?)?;
    let query_url = format!("http://{}:{}", host, query.query_port.unwrap());
    if !nodes.contains(&query_url) {
        info!("{} is missing from nodes", query_url);
        return node_add_core(conn, &query_url, req.state());
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct NodeListQuery {
    query_port: Option<u32>,
}

pub fn node_list(query: Query<NodeListQuery>, req: HttpRequest<AppState>) -> HttpResponse {
    let conn = req.state().pool.get().unwrap();
    let mut nodes = Vec::new();
    for row in conn.query("select url from nodes").expect("last select works").iter() {
        let url: String = row.get("url");
        nodes.push(url);
    }
    if let Err(err) = add_node_from_req(query, req, &nodes, &conn) {
        warn!("Error from add_node_from_req: {}", err);
    }
    HttpResponse::Ok().json(nodes)
}
