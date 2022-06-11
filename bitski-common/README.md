# bitski-common

A library for common tasks.

### Features

`bitski-common` has optional features that can be added in your `Cargo.toml`
according to your needs.

- `actix` adds support for Actix errors
- `actix-web` adds support Actix Web server and errors, see
  [rust-api-template](https://github.com/BitskiCo/rust-api-template) for an
  example
- `awc` adds support for `awc` errors
- `bcrypt` adds support for `bcrypt` errors
- `diesel` adds support for Diesel
- `humantime` enables `parse_env_duration*` methods and adds support for
  `humantime` errors
- `lettre` adds support for `lettre` errors
- `oauth2` adds support for `oauth2` errors
- `postgres` _(implies `diesel`)_ adds support for PostgreSQL
- `r2d2` adds support for `r2d2` errors
- `reqwest` adds support for `reqwest` errors
- `test` enables methods used in tests
- `tonic` adds support Tonic gRPC server
- `tower` _(implies `tonic`)_ enables Tower middleware for Tonic
- `validator` adds support for `validator` errors
