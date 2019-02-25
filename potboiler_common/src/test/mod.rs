use failure::Error;
use log::debug;
use std::{thread, time};

mod record_server;
mod server_thread;

pub use record_server::{RecordRequest, RecordServer};
pub use server_thread::ServerThread;

pub fn wait_for_action<F, R>(action: F) -> Result<R, Error>
where
    F: Fn() -> Result<R, Error>,
{
    let max = 4;
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
