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

#[derive(Copy, Clone)]
pub struct Nodes;

impl Key for Nodes {
    type Value = HashMap<String, String>;
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
        .insert(to_notify.clone(), String::from(""));
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
