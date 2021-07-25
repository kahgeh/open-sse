use std::thread::sleep;
use std::time::Duration;

pub fn is_ready () -> bool {
    let retry_interval = Duration::new(10,0);
    sleep(retry_interval);
    true
}