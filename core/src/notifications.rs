use hyper;
use iron::Request;
use iron::prelude::{IronError, IronResult, Response};
use iron::status;
use iron::typemap::Key;
use persistent;
use persistent::State;
use postgres;
use postgres::error::SqlState;
use potboiler_common::{db, url_from_body};
use potboiler_common::types::Log;
use r2d2;
use r2d2_postgres;
use serde_json;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::thread;
use url::Url;
pub type PostgresConnection = r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>;

#[derive(Copy, Clone)]
pub struct Notifications;

impl Key for Notifications {
    type Value = Vec<String>;
}

pub fn init_notifiers(conn: &PostgresConnection) -> Vec<String> {
    let mut notifiers = Vec::new();
    let stmt = conn.prepare("select url from notifications").expect("prepare failure");
    for row in &stmt.query(&[]).expect("notifications select works") {
        let url: String = row.get("url");
        notifiers.push(url);
    }
    return notifiers;
}

fn get_notifications_list(req: &Request) -> Vec<String> {
    req.extensions.get::<State<Notifications>>().unwrap().read().unwrap().deref().clone()
}

fn insert_notifier(req: &mut Request, to_notify: &String) {
    req.extensions
        .get_mut::<State<Notifications>>()
        .unwrap()
        .write()
        .unwrap()
        .deref_mut()
        .push(to_notify.clone());
}

pub fn notify_everyone(req: &Request, log_arc: Arc<Log>) {
    let notifications = get_notifications_list(req);
    for notifier in notifications {
        let local_log = log_arc.clone();
        thread::spawn(move || {
            let client = hyper::client::Client::new();
            debug!("Notifying {:?}", notifier);
            let res = client.post(&notifier)
                .body(&serde_json::ser::to_string(&local_log).unwrap())
                .send();
            match res {
                Ok(val) => {
                    if val.status != hyper::status::StatusCode::NoContent {
                        warn!("Failed to notify {:?}: {:?}", &notifier, val.status);
                    }
                }
                Err(val) => {
                    warn!("Failed to notify {:?}: {:?}", &notifier, val);
                }
            };
        });
    }
}

pub fn log_register(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let url = url_from_body(req).unwrap().unwrap();
    debug!("Registering {:?}", url);
    match Url::parse(&url) {
        Err(err) => Err(IronError::new(err, (status::BadRequest, "Bad URL"))),
        Ok(_) => {
            match conn.execute("INSERT INTO notifications (url) VALUES ($1)", &[&url]) {
                Ok(_) => {
                    insert_notifier(req, &url);
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

pub fn log_deregister(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    conn.execute("DELETE from notifications where url = $1",
                 &[&url_from_body(req).unwrap().unwrap()])
        .expect("delete worked");
    Ok(Response::with((status::NoContent)))
}
