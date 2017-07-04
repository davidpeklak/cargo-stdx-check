# cargo-stdx-check
[![Build Status](https://travis-ci.org/davidpeklak/cargo-stdx-check.svg?branch=master)](https://travis-ci.org/davidpeklak/cargo-stdx-check)
[![Latest Version](https://img.shields.io/crates/v/cargo-stdx-check.svg)](https://crates.io/crates/cargo-stdx-check)

A cargo custom command to test crates against [stdx](https://github.com/brson/stdx).
## Installation
Install cargo-stdx-check with
```
cargo install cargo-stdx-check
```
## Run as cargo custom command
Once installed, you can run cargo-stdx-check as a cargo custom commmand.
In your crate, or any crate you want to check run
 ```
 cargo stdx-check
 ```
 cargo-stdx-check will perform the following steps:
 * Backup the Cargo.toml and Cargo.lock files as Cargo.toml.bk and Cargo.lock.bk
 * Add [stdx](https://github.com/brson/stdx) as a dependency to Cargo.toml
 (from its git repository, not from crates.io)
 * Run `cargo test`
 * Check for duplicates of dependencies in Cargo.lock
 
 Note that with the [current version of stdx](https://github.com/brson/stdx/tree/f6e3c0c8dcafde3e661d31afcf86e91acd1d3166)
 the step `cargo test` will always fail, because it requires a version of the
 `chrono` crate that it cannot find.
 ## Options
 ### Help
 ```
 cargo stdx-check --help
 cargo stdx-check -h
 ```
 ### `cargo test` only
 ```
 cargo stdx-check test
 ```
 ### Check for duplicates only
 ```
 cargo stdx-check dupes
 ```
 ### Specify stdx version
 To specify a version of stdx from crates.io, run
 ```
 cargo stdx-check --stdxversion <version>
 ```
 To specify a git repository to load stdx from, run
 ```
 cargo stdx-check --stdxgit <repository>
 ```
 