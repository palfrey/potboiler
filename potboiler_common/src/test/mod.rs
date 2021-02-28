use anyhow::Result;
use log::debug;
use std::{thread, time};

mod record_server;
mod server_thread;

pub use record_server::{RecordRequest, RecordServer};
pub use server_thread::ServerThread;

pub fn wait_for_action<F, R>(action: F) -> Result<R>
where
    F: Fn() -> Result<R>,
{
    let max = 10;
    for x in 0..max {
        let res = action();
        if res.is_ok() {
            return res;
        }
        debug!("Failed on count {} with action", x);
        thread::sleep(time::Duration::from_millis(500));
        if x == max - 1 {
            // failure case
            return res;
        }
    }
    unreachable!();
}
