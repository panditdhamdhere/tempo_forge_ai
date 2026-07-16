use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempoforge_common::{AppError, AppResult};

#[derive(Clone)]
pub struct RateLimiter {
    limit: u32,
    inner: Arc<Mutex<HashMap<String, Window>>>,
}

struct Window {
    count: u32,
    started: Instant,
}

impl RateLimiter {
    pub fn new(limit_per_minute: u32) -> Self {
        Self {
            limit: limit_per_minute.max(1),
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check(&self, key: &str) -> AppResult<()> {
        let mut map = self.inner.lock();
        let now = Instant::now();
        let entry = map.entry(key.to_string()).or_insert(Window {
            count: 0,
            started: now,
        });

        if now.duration_since(entry.started) > Duration::from_secs(60) {
            entry.count = 0;
            entry.started = now;
        }

        if entry.count >= self.limit {
            return Err(AppError::RateLimited);
        }

        entry.count += 1;
        Ok(())
    }
}
