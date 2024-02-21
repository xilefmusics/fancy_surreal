# Fancy Surreal

The `fancy_surreal` rust library crate provides new abstractions to work with the official [surrealdb](https://crates.io/crates/surrealdb) crate.
It provides a trait `Databasable` and functions to write/read structs implementing this trait to/from the database.
Therefore it provides builder patterns to build queries.

## Usage

This library is not yet released to [crates.io](https://crates.io) therefore you have to clone and reference the source code directly.
An example that uses this library can be seen in the [money-app](https://github.com/xilefmusics/money-app/blob/main/docker-compose.yaml).
In the future this library will be properly released to crates.io.

## License

[![GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
