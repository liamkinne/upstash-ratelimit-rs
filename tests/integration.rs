use std::time::Duration;

use ratelimit::{Limiter, RateLimit, Response};

#[test]
fn test_add() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let ratelimit = RateLimit::builder()
        .redis(client)
        .limiter(Limiter::FixedWindow {
            tokens: 10,
            window: Duration::from_millis(1000),
        })
        .build()
        .unwrap();

    let result = ratelimit.limit("test".to_string()).unwrap();

    match result {
        Response::Success {
            limit,
            remaining,
            reset,
        } => {
            assert!(true)
        }
        Response::Failure { limit, reset } => {
            assert!(false)
        }
    }
}
