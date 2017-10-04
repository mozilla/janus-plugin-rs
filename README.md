# janus-plugin-rs

[![Build Status](https://travis-ci.org/mquander/janus-plugin-rs.svg?branch=master)](https://travis-ci.org/mquander/janus-plugin-rs)

Library for creating Rust plugins to [Janus](https://janus.conf.meetecho.com/). Still highly unstable, so not published.

``` toml
[dependencies]
janus-plugin = { git = "https://github.com/mquander/janus-plugin-rs" }
```

## Building

Requires the [Jansson](http://www.digip.org/jansson/) native library (Ubuntu: `libjansson-dev`) to link against; tested as compatible with 2.10.

```
$ cargo build --all
```

## Testing

```
$ cargo test --all
```
