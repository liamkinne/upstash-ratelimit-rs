# Redis Backed Rate Limit

This library is inspired by and designed to be compatible with the [Upstash Rate Limit](https://github.com/upstash/ratelimit) library.

## Getting Started

Add the dependency

```shell
cargo add ratelimit
```

Use ratelimit to

```rust
use ratelimit::{Limiter, RateLimit, Response};

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

## Feature Support

- [x] Single region Redis
- [ ] Multi-region Redis
- [x] Fixed window limiting
- [ ] Sliding window limiting
- [ ] Token bucket limiting
- [ ] Cached fixed window limiting
- [x] Arbitrary key prefix
- [ ] Ephemeral (in-memory) cache
- [ ] Block until ready
- [ ] HTTP Redis connection
    - This will require substantial effort and should probably reside in a separate library.
- [ ] Analytics
    - This will require substantial efford and should probably reside in a separate library.
