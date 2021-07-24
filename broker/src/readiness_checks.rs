use tracing::{error, warn, info};
use crate::settings::AppSettings;
use crate::redis_helpers::{test_redis_connection};
use std::thread::sleep;
use std::time::Duration;

pub fn is_ready (app_settings: &AppSettings)-> bool {
    let redis_connection_string = app_settings.settings.outgoing_endpoints.redis.as_str();
    let max_retry_count = app_settings.settings.readiness.retry_count;
    let retry_interval_in_seconds = app_settings.settings.readiness.retry_interval_in_seconds;
    let mut retry_count = 0;
    let retry_interval = Duration::new(retry_interval_in_seconds as u64,0);
    loop {
        if true == test_redis_connection(redis_connection_string) {
            info!("client register is ready");
            return true;
        }

        if retry_count >= max_retry_count {
            error!("attempted to reach {} {} times but failed", redis_connection_string, max_retry_count );
            return false;
        }

        if retry_count >= 1 {
            warn!("reaching {} (number of attempts so far {})...", redis_connection_string, retry_count)
        }

        sleep(retry_interval);
        retry_count=retry_count+1;
    }
}