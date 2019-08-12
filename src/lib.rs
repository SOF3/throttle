// throttle
// Copyright (C) SOFe
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Throttle is a simple utility for rate-limiting operations.
///
/// ```
/// use std::time::Duration;
/// use throttle::Throttle;
///
/// let unit = Duration::from_millis(100); // we use 100ms to have sufficient time elapse
/// let mut throttle = Throttle::new(unit * 4, 3);
///
/// throttle.accept().expect("The throttle is empty");
/// assert_eq!(throttle.size(), 1);
/// std::thread::sleep(unit * 2); // we sleep for 2t, still not expired
///
/// assert_eq!(throttle.size(), 1);
/// throttle.accept().expect("The throttle has one more space");
/// assert_eq!(throttle.size(), 2);
/// std::thread::sleep(unit); // time is now +3t
///
/// assert_eq!(throttle.size(), 2);
/// throttle.accept().expect("The last accept before throttle is full");
/// assert_eq!(throttle.size(), 3);
/// throttle.accept().expect_err("The throttle should be full");
/// assert_eq!(throttle.size(), 3);
///
/// std::thread::sleep(unit * 2); // time is now +5t, and the first accept should have expired
/// assert_eq!(throttle.size(), 2);
/// throttle.accept().expect("The first accept should have expired");
/// assert_eq!(throttle.size(), 3);
/// throttle.accept().expect_err("The second accept should not have expired yet");
/// assert_eq!(throttle.size(), 3);
///
/// std::thread::sleep(unit * 10); // time is now +10t, and all accepts should have expired
/// assert_eq!(throttle.size(), 0);
/// ```
pub struct Throttle {
    timeout: Duration,
    threshold: usize,
    deque: VecDeque<Instant>,
}

impl Throttle {
    /// Creates a new Throttle
    pub fn new(timeout: Duration, threshold: usize) -> Throttle {
        Throttle {
            timeout,
            threshold,
            deque: Default::default(),
        }
    }

    fn flush(&mut self) {
        while let Some(first) = self.deque.front() {
            if first.elapsed() >= self.timeout.clone() {
                self.deque.pop_front();
            } else {
                break;
            }
        }
    }

    /// Returns the number of remaining items in the throttle
    pub fn size(&mut self) -> usize {
        self.flush();
        self.deque.len()
    }

    /// Checks that the throttle is availbale to accept.
    ///
    /// Pay attention to race conditions. If this returns true and the current context has a
    /// mutable reference to the throttle, the availablity remains true within the context.
    /// However, if this returns false, it is possible that this becomes true in the next line.
    pub fn available(&mut self) -> bool {
        self.size() < self.threshold
    }

    /// Attempts to accept an operation and increment the throttle.
    ///
    /// On success, Ok is returned and the counter increments.
    ///
    /// On failure, Err is returned with an Instant indicating the time that the throttle is
    /// available again.
    pub fn accept(&mut self) -> Result<(), Instant> {
        self.flush();
        if self.deque.len() >= self.threshold {
            return Err(self.deque.front().unwrap().clone() + self.timeout.clone());
        }

        self.deque.push_back(Instant::now());
        Ok(())
    }
}
