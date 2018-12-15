use hyper;
use iron::{
    prelude::{IronError, IronResult, Response},
    status,
    typemap::Key,
    Request,
};
use log::{debug, warn};
use persistent::{self, State};
use potboiler_common::{db, get_db_connection, types::Log, url_from_body};
use serde_json;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    thread,
};
use url::Url;

#[derive(Copy, Clone)]
pub struct Notifications;

impl Key for Notifications {
    type Value = Vec<String>;
}

pub fn init_notifiers(conn: &db::Connection) -> Vec<String> {
    let mut notifiers = Vec::new();
    for row in &conn
        .query("select url from notifications")
        .expect("notifications select works")
    {
        let url: String = row.get("url");
        notifiers.push(url);
    }
    notifiers
}

fn get_notifications_list(req: &Request) -> Vec<String> {
    req.extensions
        .get::<State<Notifications>>()
        .unwrap()
        .read()
        .unwrap()
        .deref()
        .clone()
}

fn insert_notifier(req: &mut Request, to_notify: &str) {
    req.extensions
        .get_mut::<State<Notifications>>()
        .unwrap()
        .write()
        .unwrap()
        .deref_mut()
        .push(to_notify.to_string());
}

pub fn notify_everyone(req: &Request, log_arc: &Arc<Log>) {
    let notifications = get_notifications_list(req);
    for notifier in notifications {
        let local_log = log_arc.clone();
        thread::spawn(move || {
            let client = hyper::client::Client::new();
            debug!("Notifying {:?}", notifier);
            let res = client
                .post(&notifier)
                .body(&serde_json::to_string(&local_log.deref()).unwrap())
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
    let conn = get_db_connection!(&req);
    let url = url_from_body(req).unwrap().unwrap();
    debug!("Registering {:?}", url);
    match Url::parse(&url) {
        Err(err) => Err(IronError::new(err, (status::BadRequest, "Bad URL"))),
        Ok(_) => match conn.execute(&format!("insert into notifications (url) values('{}')", &url)) {
            Ok(_) => {
                insert_notifier(req, &url);
                Ok(Response::with(status::NoContent))
            }
            Err(db::Error(db::ErrorKind::UniqueViolation, _)) => Ok(Response::with(status::NoContent)),
            Err(err) => Err(IronError::new(err, (status::BadRequest, "Some other error"))),
        },
    }
}

pub fn log_deregister(req: &mut Request) -> IronResult<Response> {
    let conn = get_db_connection!(&req);
    conn.execute(&format!(
        "delete from notifications where url = '{}'",
        &url_from_body(req)?.unwrap()
    ))
    .expect("delete worked");
    Ok(Response::with(status::NoContent))
}
