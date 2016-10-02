use hybrid_clocks::{Timestamp, Wall, WallT};
use hybrid_clocks::Clock as HClock;
use iron::Request;
use iron::typemap::Key;
use persistent::State;
use std::ops::DerefMut;

#[derive(Copy, Clone)]
pub struct Clock;

impl Key for Clock {
    type Value = HClock<Wall>;
}

pub fn get_timestamp(req: &mut Request) -> Timestamp<WallT> {
    req.extensions.get_mut::<State<Clock>>().unwrap().write().unwrap().deref_mut().now()
}

pub fn observe_timestamp(req: &mut Request, tstamp: &Timestamp<WallT>) {
    req.extensions
        .get_mut::<State<Clock>>()
        .unwrap()
        .write()
        .unwrap()
        .deref_mut()
        .observe(tstamp)
        .unwrap();
}
