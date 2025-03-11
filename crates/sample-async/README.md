# Sample Async Package

## Core Elements
- [Tokio](https://tokio.rs): runtime and general utility
- [Futures](https://github.com/rust-lang/futures-rs)
- [Reqwest](https://github.com/seanmonstar/reqwest): HTTP requests 
  - [governor](https://docs.rs/governor/latest/governor/_guide/index.html): leaky bucket rate limiter
  - [url](https://docs.rs/url/latest/url/): url string parsing (re-exported by reqwest, but we need it for its `ParseError`)
    - warn: easy to misuse and questionably helpful
- [Sqlx](https://github.com/launchbadge/sqlx): raw SQL interface (with compile-time checks)

### Reqwest Elements

- **Client** (builder)
  - timeout
  - default headers
  - https req
  - rustls tls
  - cookie store
  - ...
- **Request** (builder)
  - request w/ Method::(GET|POST|etc)
  - header
  - auth
  - query
  - body
    - json
  - try_clone
  - ...
- **Response**
  - status
    - error_for_status (ref)
  - cookies
  - text
    - json
  - bytes stream
