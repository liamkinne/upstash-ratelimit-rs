![crates.io](https://img.shields.io/crates/v/upstash-ratelimit.svg)

# Redis Backed Rate Limit

This library is inspired by and designed to be compatible with the [Upstash Rate Limit](https://github.com/upstash/ratelimit) library. This means you can have multiple JavaScript and Rust services operate on the same Redis keys and use the same rate limiting algorithms.

Please note this library is not yet stable. The first stable version will be publushed to crates.io as v0.2.0.

## Getting Started

Add the dependency

```shell
cargo add upstash-ratelimit
```

Use upstash_ratelimit to requests

```rust
use upstash_ratelimit::{Limiter, RateLimit, Response};

let redis = redis::Client::open("redis://127.0.0.1/")?;

let ratelimit = RateLimit::builder()
    .redis(REDIS.clone())
    .limiter(Limiter::FixedWindow {
        tokens: 30,
        window: Duration::from_millis(100),
    })
    .build()?;

let response = ratelimit.limit("unique-id");

match result {
    Response::Success { .. } => println!("Allow!")
    Response::Failure { .. } => println!("Rate limited!")
}
```

## Running Unit Tests

Before running `cargo test`, spin up a Redis instance on `localhost` with `docker-compose up -d`. It's a good idea to restart Redis between test runs.

## Feature Coverage

- [x] Single region Redis
- [ ] Multi-region Redis
- [x] Fixed window limiting
- [x] Sliding window limiting
- [ ] Token bucket limiting
- [ ] Cached fixed window limiting
- [x] Arbitrary key prefix
- [ ] Ephemeral (in-memory) cache
- [ ] Block until ready
- [ ] HTTP Redis connection
    - This will require substantial effort and should probably reside in a separate library.
- [ ] Analytics
    - This will require substantial efford and should probably reside in a separate library.

## Other Tasks

- [ ] Compatibility integration testing with JS client.
