use crate::AppState;
use actix_web::{HttpResponse, Json, State};
use log::{debug, warn};
use potboiler_common::{db, types::Log};
use serde_derive::Deserialize;
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
    thread,
};
use url::Url;

#[derive(Clone, Debug)]
pub struct Notifications {
    notifiers: Arc<RwLock<Vec<String>>>,
}

impl Notifications {
    pub fn new(conn: &db::Connection) -> Notifications {
        let mut notifiers = Vec::new();
        for row in &conn
            .query("select url from notifications")
            .expect("notifications select works")
        {
            let url: String = row.get("url");
            notifiers.push(url);
        }
        Notifications {
            notifiers: Arc::new(RwLock::new(notifiers)),
        }
    }

    pub fn notify_everyone(&self, log_arc: &Arc<Log>) {
        for notifier in self.notifiers.read().unwrap().deref() {
            let local_log = log_arc.clone();
            let local_notifier = notifier.clone();
            thread::spawn(move || {
                let client = reqwest::Client::new();
                debug!("Notifying {:?}", local_notifier);
                let res = client.post(&local_notifier).json(&local_log.deref()).send();
                match res {
                    Ok(val) => {
                        if val.status() != reqwest::StatusCode::NO_CONTENT {
                            warn!("Failed to notify {:?}: {:?}", &local_notifier, val.status());
                        }
                    }
                    Err(val) => {
                        warn!("Failed to notify {:?}: {:?}", &local_notifier, val);
                    }
                };
            });
        }
    }
}

#[derive(Deserialize)]
pub struct UrlJson {
    url: String,
}

pub fn log_register(state: State<AppState>, body: Json<UrlJson>) -> HttpResponse {
    let conn = state.pool.get().unwrap();
    let url = &body.url;
    debug!("Registering {:?}", url);
    match Url::parse(&url) {
        Err(_) => HttpResponse::BadRequest().reason("Bad URL").finish(),
        Ok(_) => match conn.execute(&format!("insert into notifications (url) values('{}')", &url)) {
            Ok(_) => {
                state
                    .notifications
                    .notifiers
                    .write()
                    .unwrap()
                    .deref_mut()
                    .push(url.to_string());
                HttpResponse::Created().finish()
            }
            Err(db::Error::UniqueViolation) => HttpResponse::Created().finish(),
            Err(err) => HttpResponse::BadRequest().body(format!("Some other error: {}", err)),
        },
    }
}

pub fn log_deregister(state: State<AppState>, body: Json<UrlJson>) -> HttpResponse {
    let conn = state.pool.get().unwrap();
    let url = &body.url;
    let res = conn
        .execute(&format!("delete from notifications where url = '{}'", url))
        .expect("delete worked");
    if res == 1 {
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}
