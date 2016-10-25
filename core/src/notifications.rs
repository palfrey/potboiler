use iron::Request;
use iron::typemap::Key;
use persistent::State;
use std::ops::{Deref, DerefMut};
use std::thread;
use serde_types::Log;
use hyper;
use serde_json;
use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct Notifications;

impl Key for Notifications {
    type Value = Vec<String>;
}

fn get_notifications_list(req: &Request) -> Vec<String> {
    req.extensions.get::<State<Notifications>>().unwrap().read().unwrap().deref().clone()
}

pub fn insert_notifier(req: &mut Request, to_notify: &String) {
    req.extensions
        .get_mut::<State<Notifications>>()
        .unwrap()
        .write()
        .unwrap()
        .deref_mut()
        .push(to_notify.clone());
}

pub fn notify_everyone(req: &Request, log: Log) {
    let notifications = get_notifications_list(req);
    let log_arc = Arc::new(log);
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
                },
                Err(val) => {
                    warn!("Failed to notify {:?}: {:?}", &notifier, val);
                }
            };
        });
    }
}
