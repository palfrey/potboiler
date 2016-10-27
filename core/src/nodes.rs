use hyper;
use iron::prelude::{Request, IronError, IronResult, Response};
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

use r2d2;
use r2d2_postgres;

#[derive(Copy, Clone)]
pub struct Nodes;

pub struct NodeInfo {

}

impl Key for Nodes {
    type Value = HashMap<String, NodeInfo>;
}

fn check_host(host_url: String) {
    let sleep_time = Duration::from_secs(5);
    let client = hyper::client::Client::new();
    loop {
        let check_url = format!("{:?}/log", &host_url);
        info!("Checking {:?} ({:?})", host_url, check_url);
        let res = client.get(&check_url).send();
        match res {
            Ok(val) => {
                info!("Got value back: {:?}", val);
            }
            Err(val) => {
                warn!("Failed to get logs from {:?}: {:?}", &host_url, val);
            }
        };
        thread::sleep(sleep_time);
    }
}

pub fn initial_nodes(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> HashMap<String, NodeInfo> {
    let mut nodes = HashMap::new();
    let stmt = conn.prepare("select url from nodes").expect("prepare failure");
    for row in &stmt.query(&[]).expect("nodes select works") {
        let url: String = row.get("url");
        nodes.insert(url.clone(), NodeInfo{});
        thread::spawn(move || check_host(url.clone()));
    }
    return nodes;
}

fn get_nodes_list(req: &Request) -> Vec<String> {
    let state_ref = req.extensions.get::<State<Nodes>>().unwrap().read().unwrap();
    let state = state_ref.deref();
    let mut vec = Vec::with_capacity(state.len());
    for key in state.keys() {
        vec.push(key.clone());
    }
    return vec;
}

fn insert_node(req: &mut Request, to_notify: &String) {
    req.extensions
        .get_mut::<State<Nodes>>()
        .unwrap()
        .write()
        .unwrap()
        .deref_mut()
        .insert(to_notify.clone(), NodeInfo{});
}

pub fn notify_everyone(req: &Request, log_arc: Arc<Log>) {
    let nodes = get_nodes_list(req);
    for node in nodes {
        let local_log = log_arc.clone();
        thread::spawn(move || {
            let client = hyper::client::Client::new();
            debug!("Notifying {:?}", node);
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
    debug!("Registering node {:?}", url);
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
