use hybrid_clocks::{Clock as HClock, Timestamp, WallMS, WallMST};
use std::{
    ops::DerefMut,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct SyncClock {
    clock: Arc<RwLock<HClock<WallMS>>>,
}

impl SyncClock {
    pub fn new() -> SyncClock {
        SyncClock {
            clock: Arc::new(RwLock::new(HClock::wall_ms().unwrap())),
        }
    }

    pub fn get_timestamp(&self) -> Timestamp<WallMST> {
        self.clock.write().unwrap().deref_mut().now().unwrap()
    }

    pub fn observe_timestamp(&self, timestamp: Timestamp<WallMST>) {
        self.clock.write().unwrap().deref_mut().observe(&timestamp);
    }
}

impl Default for SyncClock {
    fn default() -> Self {
        Self::new()
    }
}
