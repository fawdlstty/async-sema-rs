mod utils;

use chrono::NaiveDateTime;
use event_listener::Event;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use utils::time_util::NaiveDateTimeExt;

#[cfg(test)]
mod test;

/// A counter for limiting the number of concurrent operations.
#[derive(Debug)]
pub struct Semaphore {
    count: AtomicUsize,
    event: Event,
}

impl Semaphore {
    /// Creates a new semaphore with a limit of `n` concurrent operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_semaphore::Semaphore;
    ///
    /// let s = Semaphore::new(5);
    /// ```
    pub const fn new(n: usize) -> Semaphore {
        Semaphore {
            count: AtomicUsize::new(n),
            event: Event::new(),
        }
    }

    /// Attempts to get a permit for a concurrent operation.
    ///
    /// If the permit could not be acquired at this time, then [`None`] is returned. Otherwise, a
    /// guard is returned that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    ///
    /// let s = Semaphore::new(2);
    ///
    /// s.try_acquire().unwrap();
    /// s.try_acquire().unwrap();
    ///
    /// assert!(s.try_acquire().is_none());
    /// s.add_permits(1);
    /// assert!(s.try_acquire().is_some());
    /// ```
    pub fn try_acquire(&self) -> bool {
        let mut count = self.count.load(Ordering::Acquire);
        loop {
            if count == 0 {
                return false;
            }

            match self.count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(c) => count = c,
            }
        }
    }

    /// Waits for a permit for a concurrent operation.
    ///
    /// Returns a guard that releases the permit when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    ///
    /// let s = Semaphore::new(2);
    /// s.acquire().await;
    /// ```
    pub async fn acquire(&self) {
        let mut listener = None;

        loop {
            if self.try_acquire() {
                return;
            }

            match listener.take() {
                None => listener = Some(self.event.listen()),
                Some(l) => l.await,
            }
        }
    }

    /// Waits for a permit for a concurrent operation.
    ///
    /// Returns a guard that releases the permit when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    ///
    /// let s = Semaphore::new(2);
    /// s.acquire().await;
    /// ```
    pub async fn acquire_timeout(&self, dur: Duration) -> bool {
        let elapsed = Arc::new(AtomicBool::new(false));
        let elapsed2 = Arc::clone(&elapsed);
        let fut = async move {
            self.acquire().await;
            if elapsed2.load(Ordering::SeqCst) {
                self.add_permits(1);
            }
        };
        match timeout(dur, fut).await {
            Ok(_) => true,
            Err(_) => {
                elapsed.store(true, Ordering::SeqCst);
                false
            }
        }
    }

    /// Add permit for a concurrent operations
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    ///
    /// let s = Semaphore::new(0);
    /// assert!(s.try_acquire().is_none());
    /// s.add_permits(1);
    /// assert!(s.try_acquire().is_some());
    /// ```
    pub fn add_permits(&self, n: usize) {
        self.count.fetch_add(n, Ordering::AcqRel);
        self.event.notify(n);
    }
}
