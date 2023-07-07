#![allow(unused)]

pub mod builder;

use anyhow::{anyhow, Result};
use redis::{Client as RedisClient, Script};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub enum Limiter {
    /// Each request inside a fixed time increases as counter. Once the
    /// counter reaches the maximum allowed number, all futher requetss are
    /// rejected.
    FixedWindow {
        /// How many requests are allowed per window.
        tokens: u64,
        /// Duration in which the request limit is replied.
        window: Duration,
    },
    ///
    SlidingLogs {
        /// How many requests are allowed per window.
        tokens: u64,
        /// Duration in which the request limit is replied.
        window: Duration,
    },
    /// Sliding version of `fixedWindow` with lower storage width improved
    /// boundary behavior by calculating a weighted score between two windows.
    SlidingWindow {
        /// How many requests are allowed per window.
        tokens: u64,
        /// Duration in which the request limit is replied.
        window: Duration,
    },
    /// Each bucket is filled with `max_tokens` that refills until full at a
    /// rate of `refill_rate` per `interval`. Every request will remove one
    /// token from the bucket and if there is no tokens left, the request is
    /// rejected.
    TokenBucket {
        /// How many requests are allowed per window.
        refill_rate: u64,
        /// The interval for the refill rate.
        interval: Duration,
        /// Maximum number of tokens.
        ///
        /// A newly created bucket starts with this many tokens.
        max_tokens: u64,
    },
}

pub enum Response {
    Success {
        /// Remaining tokens before rate limiting is applied.
        remaining: u64,
        /// Amount of time until rate limit is lifted.
        reset: Duration,
    },
    Failure {
        /// Amount of time until rate limit is lifted.
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

    /// Get current system time
    fn now(&self) -> Result<Duration> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?)
    }

    /// Apply limiting based on a given unique identifier.
    ///
    /// The identifier could be a user id, IP address or any other string.
    pub fn limit<T>(&self, identifier: T) -> Result<Response>
    where
        T: Into<String> + std::fmt::Display,
    {
        return match self.limiter {
            Limiter::FixedWindow { tokens, window } => {
                let script = Script::new(
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
                let now = self.now()?.as_millis();
                let bucket = now / window_duration;
                let key = format!("{}:{}", identifier, bucket);

                // todo: implement in-memory cache

                let mut redis = self.redis.get_connection()?;
                let tokens_used: u64 = script
                    .key(key)
                    .arg(format!("{}", window_duration))
                    .invoke(&mut redis)?;

                let reset = Duration::from_millis(((bucket + 1) * window_duration) as u64);

                if tokens_used <= tokens {
                    Ok(Response::Success {
                        remaining: tokens - tokens_used,
                        reset,
                    })
                } else {
                    Ok(Response::Failure { reset })
                }
            }
            Limiter::SlidingWindow { tokens, window } => {
                let script = Script::new(
                    r#"
                        local currentKey  = KEYS[1]           -- identifier including prefixes
                        local previousKey = KEYS[2]           -- key of the previous bucket
                        local tokens      = tonumber(ARGV[1]) -- tokens per window
                        local now         = ARGV[2]           -- current timestamp in milliseconds
                        local window      = ARGV[3]           -- interval in milliseconds

                        local requestsInCurrentWindow = redis.call("GET", currentKey)
                        if requestsInCurrentWindow == false then
                        requestsInCurrentWindow = -1
                        end


                        local requestsInPreviousWindow = redis.call("GET", previousKey)
                        if requestsInPreviousWindow == false then
                        requestsInPreviousWindow = 0
                        end
                        local percentageInCurrent = ( now % window) / window
                        if requestsInPreviousWindow * ( 1 - percentageInCurrent ) + requestsInCurrentWindow >= tokens then
                        return -1
                        end

                        local newValue = redis.call("INCR", currentKey)
                        if newValue == 1 then
                        -- The first time this key is set, the value will be 1.
                        -- So we only need the expire command once
                        redis.call("PEXPIRE", currentKey, window * 2 + 1000) -- Enough time to overlap with a new window + 1 second
                        end
                        return tokens - newValue
                    "#,
                );

                let window_duration = window.as_millis();

                let now = self.now()?.as_millis();
                let current_window = now / window_duration;
                let current_key = format!("{}:{}", identifier, current_window);
                let previous_window = current_window - window_duration;
                let previous_key = format!("{}:{}", identifier, previous_window);

                // todo: implement in-memory chache

                let mut redis = self.redis.get_connection()?;

                let remaining: i64 = script
                    .key(current_key)
                    .key(previous_key)
                    .arg(tokens)
                    .arg(now as u64)
                    .arg(window_duration as u64)
                    .invoke(&mut redis)?;

                let reset = Duration::from_millis(((current_window + 1) * window_duration) as u64);

                if remaining < 0 {
                    Ok(Response::Failure { reset })
                } else {
                    Ok(Response::Success {
                        remaining: remaining.max(0_i64) as u64,
                        reset,
                    })
                }
            }
            _ => unimplemented!("Limiter method not implemented."),
        };
    }
}
