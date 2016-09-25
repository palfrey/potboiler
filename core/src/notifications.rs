use iron::Request;
use iron::typemap::Key;
use persistent::State;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
pub struct Notifications;

impl Key for Notifications {
    type Value = Vec<String>;
}

pub fn get_notifications_list(req: &Request) -> Vec<String> {
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
