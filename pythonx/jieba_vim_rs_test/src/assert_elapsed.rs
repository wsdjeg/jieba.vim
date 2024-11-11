use std::time::{Duration, Instant};

pub struct AssertElapsed {
    max_duration: Duration,
    start: Instant,
}

impl AssertElapsed {
    pub fn tic(millis: u64) -> Self {
        Self {
            max_duration: Duration::from_millis(millis),
            start: Instant::now(),
        }
    }

    pub fn toc(&self) {
        let duration = self.start.elapsed();
        assert!(duration <= self.max_duration);
    }
}
