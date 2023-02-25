use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn current_unix() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}
