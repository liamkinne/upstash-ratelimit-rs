# Redis Backed Rate Limit

This library is inspired by and designed to be compatible with the [Upstash Rate Limit](https://github.com/upstash/ratelimit) library.

## Feature Support

- [x] Single region Redis
- [ ] Multi-region Redis
- Limiting methods:
    - [x] Fixed window
    - [ ] Sliding window
    - [ ] Token bucket
    - [ ] Cached fixed window
- [x] Arbitrary key prefix
- [ ] Ephemeral (in-memory) cache
- [ ] Block until ready
- [ ] Analytics
