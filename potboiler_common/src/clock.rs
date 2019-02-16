use hybrid_clocks::{Clock as HClock, Timestamp, Wall, WallT};
use std::{
    ops::DerefMut,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct SyncClock {
    clock: Arc<RwLock<HClock<Wall>>>,
}

impl SyncClock {
    pub fn new() -> SyncClock {
        SyncClock {
            clock: Arc::new(RwLock::new(HClock::wall())),
        }
    }

    pub fn get_timestamp(&self) -> Timestamp<WallT> {
        self.clock.write().unwrap().deref_mut().now()
    }

    pub fn observe_timestamp(&self, timestamp: Timestamp<WallT>) {
        self.clock.write().unwrap().deref_mut().observe(&timestamp).unwrap();
    }
}
