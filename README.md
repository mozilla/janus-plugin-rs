# janus-plugin-rs

[![Documentation](https://docs.rs/janus-plugin/badge.svg)](https://docs.rs/janus-plugin/)
[![janus-plugin](https://img.shields.io/crates/v/janus-plugin.svg)](https://crates.io/crates/janus-plugin)
[![Build Status](https://travis-ci.org/mozilla/janus-plugin-rs.svg?branch=master)](https://travis-ci.org/mozilla/janus-plugin-rs)

Library for creating Rust plugins and event handlers for [Janus](https://janus.conf.meetecho.com/). Still moderately unstable.

``` toml
[dependencies]
janus-plugin = "0.13.0"
```

## Compatibility

Currently compatible with Janus versions >= 0.10.9; Janus makes breaking changes relatively frequently to
the plugin API, so expect this library to require updating and recompilation for plugins to continue to work with new
Janus versions.

## Building

Requires the [Jansson](http://www.digip.org/jansson/) native library (Ubuntu: `libjansson-dev`) to link against; tested as compatible with versions >= 2.5.

```
$ cargo build --all
```

## Testing

```
$ cargo test --all
```

## Basic usage

Janus expects to dynamically link plugins as libraries and then call a `create` function on them to return a
`janus_plugin` struct, which has a variety of function pointers that Janus will call when plugin-related events in the
core happen.

These bindings provide a `build_plugin!` macro that accepts as arguments plugin metadata and a set of (`extern C`) Rust
functions, producing a Rust version of the `janus_plugin` struct, and an `export_plugin!` macro that defines the
`create` function to return that struct. So to implement a plugin, you should write some handler functions, and then use
those macros like so:

``` Rust
use std::os::raw::c_char;

// helper macro for generating C-style strings from Rust string literals at compile time
macro_rules! c_str {
    ($lit:expr) => {
        unsafe {
            std::ffi::CStr::from_ptr(concat!($lit, "\0").as_ptr() as *const c_char)
        }
    }
}

extern "C" fn init(callbacks: *mut PluginCallbacks, config_path: *const c_char) -> c_int {
    janus_info!("Plugin loaded!");
    0
}

extern "C" fn destroy() {
    janus_info!("Plugin destroyed!");
}

// ...other handlers omitted: see
// https://janus.conf.meetecho.com/docs/plugin_8h.html#details

const PLUGIN: Plugin = build_plugin!(
    LibraryMetadata {
        // The Janus plugin API version. The version compiled into the plugin
        // must be identical to the version in the Janus which loads the plugin.
        api_version: 15,
        // Incrementing plugin version number for your own use.
        version: 1,
        // Human-readable metadata which Janus can query.
        name: c_str!("My plugin name"),
        package: c_str!("My plugin package name"),
        version_str: c_str!(env!("CARGO_PKG_VERSION")),
        description: c_str!(env!("CARGO_PKG_DESCRIPTION")),
        author: c_str!(env!("CARGO_PKG_AUTHORS")),
    },
    init,
    destroy,
    // ...other handlers omitted: see
    // https://janus.conf.meetecho.com/docs/plugin_8h.html#details
);

export_plugin!(&PLUGIN);
```

## Examples

Here are some projects which are using these bindings:

* https://github.com/mozilla/janus-plugin-sfu
* https://github.com/ivanovaleksey/janus-echotest-rs
* https://github.com/netology-group/janus-conference/
