# Breaker Roadmap

## 0.1.0
- [X] Router
- [X] Route
- [X] segments
    - [X] named
    - [X] wildcard
    - [X] constant
    - [ ] segment priority
- [X] match routes
    - [X] partial constant matches (first n characters)
    - [X] named and wildcard matches

## Features
- [X] constant segments
- [X] named segments
    - matches anything
    - can be in the middle or the end of a path
- [X] wildcard segments
    - match anything (zero or more, one or more)
    - must be at the end of the path
- [ ] embedded wildcards and named segments
    - used like regex (i.e. /:filename.:ext || the period is constant)
- [ ] path priority
    - constant > named > wildcard > dots and slashes
- [X] partial constant matching
    - match first n characters

## API
- router storing the routes (const generics)
- route of segments (max segment capacity)

## Long-term Goals
- [X] no alloc crate
- [ ] express simple regex in segments
