use event_listener::Event;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct SemaphoreInner {
    count: AtomicUsize,
    event: Event,
}

impl SemaphoreInner {
    pub const fn new(n: usize) -> Self {
        Self {
            count: AtomicUsize::new(n),
            event: Event::new(),
        }
    }

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

    pub fn add_permits(&self, n: usize) {
        self.count.fetch_add(n, Ordering::AcqRel);
        self.event.notify(n);
    }
}

/// A counter for limiting the number of concurrent operations.
#[derive(Debug, Clone)]
pub struct Semaphore {
    inner: Arc<SemaphoreInner>,
}

unsafe impl Send for Semaphore {}

impl Semaphore {
    /// Creates a new semaphore with a limit of `n` concurrent operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    ///
    /// let s = Semaphore::new(5);
    /// ```
    pub fn new(n: usize) -> Semaphore {
        Semaphore {
            inner: Arc::new(SemaphoreInner::new(n)),
        }
    }

    /// Attempts to get a permit for a concurrent operation.
    ///
    /// Return whether permit has been acquired
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    ///
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let s = Semaphore::new(2);
    ///
    /// s.acquire().await;
    /// s.acquire().await;
    ///
    /// assert!(!s.try_acquire());
    /// s.add_permits(1);
    /// assert!(s.try_acquire());
    /// # });
    /// ```
    pub fn try_acquire(&self) -> bool {
        self.inner.try_acquire()
    }

    /// Waits for a permit for a concurrent operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    ///
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let s = Semaphore::new(2);
    ///
    /// s.acquire().await;
    /// # });
    /// ```
    pub async fn acquire(&self) {
        self.inner.acquire().await
    }

    /// Add permit for a concurrent operations
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    ///
    /// let s = Semaphore::new(0);
    ///
    /// assert!(!s.try_acquire());
    /// s.add_permits(1);
    /// assert!(s.try_acquire());
    /// ```
    pub fn add_permits(&self, n: usize) {
        self.inner.add_permits(n)
    }
}
