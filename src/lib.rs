#![allow(unused)]

pub mod builder;

use anyhow::{anyhow, Result};
use redis::Client as RedisClient;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub enum Limiter {
    FixedWindow {
        tokens: u64,
        window: Duration,
    },
    SlidingLogs {
        tokens: u64,
        window: Duration,
    },
    SlidingWindow {
        tokens: u64,
        window: Duration,
    },
    TokenBucket {
        refill_rate: u64,
        interval: Duration,
        max_tokens: u64,
    },
}

pub enum Response {
    Success {
        limit: u64,
        remaining: u64,
        reset: Duration,
    },
    Failure {
        limit: u64,
        reset: Duration,
    },
}

pub struct RateLimit {
    redis: RedisClient,
    limiter: Limiter,
    prefix: String,
    timeout: Option<Duration>,
    analytics: bool,
}

impl RateLimit {
    /// Creates a RateLimit builder instance.
    pub fn builder() -> builder::RateLimitBuilder {
        builder::RateLimitBuilder::default()
    }

    /// Apply limiting based on a given unique identifier.
    ///
    /// The identifier could be a user id,
    pub fn limit<T>(&self, identifier: T) -> Result<Response>
    where
        T: Into<String> + std::fmt::Display,
    {
        return match self.limiter {
            Limiter::FixedWindow { tokens, window } => {
                let script = redis::Script::new(
                    r#"
                    local key     = KEYS[1]
                    local window  = ARGV[1]

                    local r = redis.call("INCR", key)
                    if r == 1 then
                    -- The first time this key is set, the value will be 1.
                    -- So we only need the expire command once
                    redis.call("PEXPIRE", key, window)
                    end

                    return r
                "#,
                );

                let window_duration = window.as_millis();

                let since_epoch = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();
                let bucket = since_epoch / window_duration;
                let key = format!("{}:{}", identifier, bucket);

                // todo: implement in-memory chache

                let mut redis = self.redis.get_connection()?;
                let tokens_used: u64 = script
                    .key(key)
                    .arg(format!("{}", window_duration))
                    .invoke(&mut redis)?;

                let reset = Duration::from_millis(((bucket + 1) * window_duration) as u64);

                if tokens_used <= tokens {
                    Ok(Response::Success {
                        limit: tokens,
                        remaining: tokens - tokens_used,
                        reset,
                    })
                } else {
                    Ok(Response::Failure {
                        limit: tokens,
                        reset,
                    })
                }
            }
            _ => unimplemented!("Limiter method not implemented."),
        };
    }
}
