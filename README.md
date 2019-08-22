# janus-plugin-rs

[![Documentation](https://docs.rs/janus-plugin/badge.svg)](https://docs.rs/janus-plugin/)
[![janus-plugin](https://img.shields.io/crates/v/janus-plugin.svg)](https://crates.io/crates/janus-plugin)
[![Build Status](https://travis-ci.org/mozilla/janus-plugin-rs.svg?branch=master)](https://travis-ci.org/mozilla/janus-plugin-rs)

Library for creating Rust plugins and event handlers for [Janus](https://janus.conf.meetecho.com/). Still moderately unstable.

``` toml
[dependencies]
janus-plugin = "0.11.1"
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

## Examples

Here are some projects which are using these bindings:

* https://github.com/mozilla/janus-plugin-sfu
* https://github.com/ivanovaleksey/janus-echotest-rs
