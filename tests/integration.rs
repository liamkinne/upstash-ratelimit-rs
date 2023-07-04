use ratelimit::{Limiter, RateLimit, Response};
use std::time::Duration;

lazy_static::lazy_static! {
    static ref REDIS: redis::Client = redis::Client::open("redis://127.0.0.1/").unwrap();
}

#[test]
fn test_success() {
    let ratelimit = RateLimit::builder()
        .redis(REDIS.clone())
        .limiter(Limiter::FixedWindow {
            tokens: 5,
            window: Duration::from_millis(10),
        })
        .build()
        .unwrap();

    let result = ratelimit.limit("test".to_string()).unwrap();

    match result {
        Response::Success { .. } => {
            assert!(true)
        }
        Response::Failure { .. } => {
            assert!(false)
        }
    }
}
