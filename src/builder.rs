use std::time::Duration;

use redis::Client as RedisClient;

use crate::{Limiter, RateLimit};

#[derive(Default)]
pub struct RateLimitBuilder {
    redis: Option<RedisClient>,
    limiter: Option<Limiter>,
    prefix: String,
    timeout: Option<Duration>,
    analytics: bool,
}

impl RateLimitBuilder {
    /// Add a redis client.
    pub fn redis(mut self, redis: RedisClient) -> RateLimitBuilder {
        self.redis = Some(redis);
        self
    }

    /// Enable analytics to get a better understanding of hour your
    /// ratelimiting is performing.
    ///
    /// See the
    /// [Upstash ratelimit documentaiton](https://github.com/upstash/ratelimit/tree/main#analytics)
    /// for more details.
    pub fn analytics(mut self, enabled: bool) -> RateLimitBuilder {
        self.analytics = enabled;
        self
    }

    /// Optionally allow request to pass after a given amount of time.
    ///
    /// Use this to allow requests through in case of a network failure.
    pub fn timeout(mut self, timeout: std::time::Duration) -> RateLimitBuilder {
        self.timeout = Some(timeout);
        self
    }

    /// Add an optional prefix to the keys used in redis.
    ///
    /// This is useful if you want to share a redis instance with other
    /// applications and you also want to avoid key collisions.
    ///
    /// The default prefix is "@upstash/ratelimit" to provide compatiblilty
    /// with the Upstash JavaScript rate limit library.
    pub fn prefix(mut self, prefix: String) -> RateLimitBuilder {
        self.prefix = prefix;
        self
    }

    pub fn build(self) -> Result<RateLimit, &'static str> {
        // apply default prefix if none is specified
        let prefix = match self.prefix.is_empty() {
            true => self.prefix,
            false => "@upstash/ratelimit".to_string(),
        };

        let redis = match self.redis {
            Some(r) => r,
            None => return Err("No redis client was defined."),
        };

        let limiter = match self.limiter {
            Some(l) => l,
            None => return Err("No limiter was defined."),
        };

        Ok(RateLimit {
            redis,
            limiter,
            prefix,
            timeout: self.timeout,
            analytics: self.analytics,
        })
    }
}
