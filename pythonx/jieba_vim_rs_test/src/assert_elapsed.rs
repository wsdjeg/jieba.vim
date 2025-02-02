// Copyright 2024 Kaiwen Wu. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

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
