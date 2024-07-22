use event_listener::Event;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time as tktime;

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

    pub async fn acquire_timeout(&self, dur: Duration) -> bool {
        let processed = Arc::new(AtomicBool::new(false));
        let processed2 = Arc::clone(&processed);
        macro_rules! mark_process {
            ($ela:expr) => {
                $ela.compare_exchange_weak(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            };
        }
        let fut = async move {
            self.acquire().await;
            if !mark_process!(processed2) {
                self.add_permits(1);
            }
        };
        match tktime::timeout(dur, fut).await {
            Ok(_) => true,
            Err(_) => !mark_process!(processed),
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

    /// Waits for a permit for a concurrent operation.
    ///
    /// Return whether permit has been acquired
    ///
    /// # Examples
    ///
    /// ```
    /// use async_sema::Semaphore;
    /// use std::time::Duration;
    ///
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let s = Semaphore::new(2);
    ///
    /// s.acquire_timeout(Duration::from_secs(1)).await;
    /// # });
    /// ```
    pub async fn acquire_timeout(&self, dur: Duration) -> bool {
        self.inner.acquire_timeout(dur).await
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
