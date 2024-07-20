# async-sema-rs

Async semaphore library

example:

```rust
use async_sema::Semaphore;

let s = Semaphore::new(2);

s.try_acquire().unwrap();
s.try_acquire().unwrap();

assert!(s.try_acquire().is_none());
s.add_permits(1);
assert!(s.try_acquire().is_some());
```
