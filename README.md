# async-sema-rs

![version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Ffawdlstty%2Fasync-sema-rs%2Fmain%2FCargo.toml&query=package.version&label=version)
![status](https://img.shields.io/github/actions/workflow/status/fawdlstty/async-sema-rs/rust.yml)

Async semaphore library

## Manual

Install: Run `cargo add async-sema` in the project directory

## example

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
