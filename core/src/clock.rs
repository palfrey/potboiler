use hybrid_clocks::{Timestamp, Wall, WallT};
use hybrid_clocks::Clock as HClock;
use iron::{Request, BeforeMiddleware, IronResult};
use iron::typemap::Key;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::RwLock;
pub type SyncClock = Arc<RwLock<HClock<Wall>>>;

#[derive(Copy, Clone)]
pub struct Clock;

impl Key for Clock {
    type Value = SyncClock;
}

#[derive(Debug)]
pub struct ClockMiddleware {
    pub clock_state: SyncClock,
}

impl BeforeMiddleware for ClockMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        debug!("self: {:?}", self);
        req.extensions.insert::<Clock>(self.clock_state.clone());
        return Ok(());
    }
}

pub fn init_clock() -> ClockMiddleware {
    let clock_state = Arc::new(RwLock::new(HClock::wall()));
    ClockMiddleware { clock_state: clock_state }
}

pub fn get_timestamp(req: &mut Request) -> Timestamp<WallT> {
    get_timestamp_from_state(req.extensions.get_mut::<Clock>().expect("get clock"))
}

pub fn get_timestamp_from_state(clock: &Arc<RwLock<HClock<Wall>>>) -> Timestamp<WallT> {
    clock.write().unwrap().deref_mut().now()
}

pub fn get_raw_timestamp(timestamp: &Timestamp<WallT>) -> Vec<u8> {
    let mut raw_timestamp: Vec<u8> = Vec::new();
    timestamp.write_bytes(&mut raw_timestamp).unwrap();
    return raw_timestamp;
}

pub fn observe_timestamp(clock_state: &Arc<RwLock<HClock<Wall>>>, timestamp: Timestamp<WallT>) {
    clock_state.write()
        .unwrap()
        .deref_mut()
        .observe(&timestamp)
        .unwrap();
}
