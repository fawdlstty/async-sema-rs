# async-sema-rs

Async semaphore library

example:

```rust
use async_sema::Semaphore;

let s = Semaphore::new(2);

// async acquire
s.acquire().await;

// instant acquire
let a = s.try_acquire().unwrap();

// async timeout acquire
let b = s.acquire_timeout(Duration::from_secs(1)).await;

assert!(s.try_acquire().is_none());
s.add_permits(1);
assert!(s.try_acquire().is_some());
```
