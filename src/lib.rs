use std::time::Instant;
use std::sync::Mutex;

pub struct RateLimiter {
    // how many new accesses are granted per second
    rate: f64,
    accesses_left: u64,
    previous_access_time: Instant,
}

impl RateLimiter {
    pub fn new(rate: f64) -> Self {
        Self {
            rate,
            accesses_left: rate as u64,
            previous_access_time: Instant::now(),
        }
    }

    pub fn has_access(&mut self) -> bool {
        let new_time = Instant::now();
        let delta = new_time - self.previous_access_time;

        self.accesses_left += (self.rate * (delta.as_nanos() as f64 / 1_000_000_000.)) as u64;
        self.previous_access_time = new_time;

        let rate = self.rate as u64;
        if self.accesses_left > rate{ self.accesses_left = rate }

        if self.accesses_left > 0 {
            self.accesses_left -= 1;
            true
        } else {
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::RateLimiter;
    use std::thread::sleep;
    use std::time::Duration;
    use std::sync::{Mutex, Arc};

    fn exhaust_rate_limiter(limiter: &mut RateLimiter) {
        for _ in 0..100 {
            assert!(limiter.has_access());
        }
        assert!(!limiter.has_access());
    }

    #[test]
    fn access() {
        let mut limiter = RateLimiter::new(100.);
        exhaust_rate_limiter(&mut limiter);

        // test that tickets regenerate
        sleep(Duration::from_secs(1));
        exhaust_rate_limiter(&mut limiter);

        // test that tickets do not build up past 100
        sleep(Duration::from_secs(2));
        exhaust_rate_limiter(&mut limiter);
    }

    #[test]
    fn access_multi_threaded() {
        let limiter = Arc::new(Mutex::new(RateLimiter::new(100.)));
        let limiter_1 = limiter.clone();

        let thread = std::thread::spawn(move || {
            for _ in 0..50 { assert!(limiter_1.lock().unwrap().has_access()); }
        });
        for _ in 0..50 { assert!(limiter.lock().unwrap().has_access()); }
        thread.join().unwrap();

        assert!(!limiter.lock().unwrap().has_access());
    }
}
