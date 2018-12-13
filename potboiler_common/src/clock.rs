use hybrid_clocks::Clock as HClock;
use hybrid_clocks::{Timestamp, Wall, WallT};
use iron::typemap::Key;
use iron::{BeforeMiddleware, IronResult, Request};
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::RwLock;
pub type SyncClock = Arc<RwLock<HClock<Wall>>>;

#[derive(Copy, Clone, Debug)]
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
        req.extensions.insert::<Clock>(self.clock_state.clone());
        Ok(())
    }
}

pub fn init_clock() -> ClockMiddleware {
    let clock_state = Arc::new(RwLock::new(HClock::wall()));
    ClockMiddleware { clock_state }
}

pub fn get_clock(req: &mut Request) -> Arc<RwLock<HClock<Wall>>> {
    req.extensions.get::<Clock>().expect("get clock").clone()
}

pub fn get_timestamp(req: &mut Request) -> Timestamp<WallT> {
    get_timestamp_from_state(&get_clock(req))
}

pub fn get_timestamp_from_state(clock: &Arc<RwLock<HClock<Wall>>>) -> Timestamp<WallT> {
    clock.write().unwrap().deref_mut().now()
}

pub fn observe_timestamp(clock_state: &Arc<RwLock<HClock<Wall>>>, timestamp: Timestamp<WallT>) {
    clock_state.write().unwrap().deref_mut().observe(&timestamp).unwrap();
}
