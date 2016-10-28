use hyper;
use iron::prelude::{IronError, IronResult, Request, Response};
use iron::status;
use iron::typemap::Key;
use persistent;
use persistent::State;
use postgres;
use postgres::error::SqlState;
use potboiler_common::{db, url_from_body};
use serde_json;
use serde_types::Log;

use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::thread;
use url::Url;
use std::collections::HashMap;
use std::time::Duration;
use std::io::Read;
use uuid::Uuid;

use r2d2;
use r2d2_postgres;
pub type PostgresConnection = r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>;
pub type PostgresPool = r2d2::Pool<r2d2_postgres::PostgresConnectionManager>;

#[derive(Copy, Clone)]
pub struct Nodes;

pub struct NodeInfo {
}

pub struct NodeList {
    nodes: HashMap<String, NodeInfo>,
    pool: PostgresPool,
}

impl Key for Nodes {
    type Value = NodeList;
}

fn check_host_once(host_url: &String,
                   raw_result: Result<hyper::client::Response, hyper::Error>,
                   conn: &PostgresConnection) {
    let mut res = match raw_result {
        Ok(val) => val,
        Err(val) => {
            warn!("Failed to get logs from {:?}: {:?}", &host_url, val);
            return ();
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
        Err(_) => {
            warn!("Got crappy data back: {:?}", body_string);
            return ();
        }
    };
    let kv = match json.as_object() {
        Some(val) => val,
        None => {
            warn!("Info isn't a map: {:?}", json);
            return ();
        }
    };
    let stmt = conn.prepare("SELECT 1 from log where id=$1")
        .expect("prepare failure");
    for (_, value) in kv.iter() {
        match Uuid::parse_str(value.as_str().unwrap()) {
            Ok(val) => {
                let results = stmt.query(&[&val]).expect("bad query");
                if results.is_empty() {
                    info!("Don't have {} locally yet", val)
                } else {
                    debug!("Already have {} locally", val);
                }
            }
            Err(_) => {
                warn!("Key {} isn't a UUID!", value);
            }
        };
    }
}

fn check_host(host_url: String, conn: PostgresConnection) {
    let sleep_time = Duration::from_secs(5);
    loop {
        let client = hyper::client::Client::new();
        let check_url = format!("{}/log", &host_url);
        info!("Checking {} ({})", host_url, check_url);
        let res = client.get(&check_url).send();
        check_host_once(&host_url, res, &conn);
        thread::sleep(sleep_time);
    }
}

pub fn initial_nodes(pool: PostgresPool) -> NodeList {
    let conn = pool.get().unwrap();
    let mut nodes = HashMap::new();
    let stmt = conn.prepare("select url from nodes").expect("prepare failure");
    for row in &stmt.query(&[]).expect("nodes select works") {
        let url: String = row.get("url");
        nodes.insert(url.clone(), NodeInfo {});
        let conn = pool.get().unwrap();
        thread::spawn(move || check_host(url.clone(), conn));
    }
    return NodeList {
        nodes: nodes,
        pool: pool,
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
    let conn = {
        let mut nodelist = req.extensions
            .get_mut::<State<Nodes>>()
            .unwrap()
            .write()
            .unwrap();
        let nodelist_dm = nodelist.deref_mut();
        nodelist_dm.nodes.insert(to_notify.clone(), NodeInfo {});
        nodelist_dm.pool.get().unwrap()
    };

    let url = to_notify.clone();
    thread::spawn(move || check_host(url, conn));
}

pub fn notify_everyone(req: &Request, log_arc: Arc<Log>) {
    let nodes = get_nodes_list(req);
    for node in nodes {
        let local_log = log_arc.clone();
        thread::spawn(move || {
            let client = hyper::client::Client::new();
            debug!("Notifying (node) {}", node);
            let res = client.post(&node)
                .body(&serde_json::ser::to_string(&local_log).unwrap())
                .send();
            match res {
                Ok(val) => {
                    if val.status != hyper::status::StatusCode::NoContent {
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
