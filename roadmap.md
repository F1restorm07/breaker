# Breaker Roadmap

## 0.1.0
- [ ] Router
- [ ] Route
- [ ] segments
    - [ ] named
    - [ ] wildcard
    - [ ] constant
    - [ ] priority
- [ ] compressed routes
- [ ] match routes

## Features
- [ ] constant segments
- [ ] named segments
    - matches anything
    - can be in the middle or the end of a path
- [ ] wildcard segments
    - match anything (zero or more, one or more)
    - must be at the end of the path
- [ ] embedded wildcards and named segments
    - used like regex (i.e. /:filename.:ext || the period is constant)
- [ ] path priority
    - constant > named > wildcard > dots and slashes

## API
- router storing the routes (const generics)
- route of segments (max segment capacity)

## Long-term Goals
- [ ] no alloc crate
- [ ] express simple regex in segments
