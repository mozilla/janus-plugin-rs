# janus-plugin-rs

[![Documentation](https://docs.rs/janus-plugin/badge.svg)](https://docs.rs/janus-plugin/)
[![janus-plugin](https://img.shields.io/crates/v/janus-plugin.svg)](https://crates.io/crates/janus-plugin)
[![Build Status](https://travis-ci.org/mquander/janus-plugin-rs.svg?branch=master)](https://travis-ci.org/mquander/janus-plugin-rs)

Library for creating Rust plugins to [Janus](https://janus.conf.meetecho.com/). Still highly unstable.

``` toml
[dependencies]
janus-plugin = "0.3.0"
```

## Building

Requires the [Jansson](http://www.digip.org/jansson/) native library (Ubuntu: `libjansson-dev`) to link against; tested as compatible with versions >= 2.5.

```
$ cargo build --all
```

## Testing

```
$ cargo test --all
```
