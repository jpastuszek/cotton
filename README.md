[![Latest Version]][crates.io] [![Documentation]][docs.rs] ![License]

"Batteries included" prelude with crates, types and functions useful for writing command-line interface tools.

This prelude aims to be useful in generic context of CLI tools and will try to minimise dependencies.

Things that fit this prelude:
* argument parsing,
* I/O including reading from stdin,
* common file operations and directory structure,
* logging,
* executing commands,
* extensions to stdlib and language functionality,
* digests and checksums,
* time and duration.

Things that will not be included:
* JSON parser or other formats,
* HTTP client or specific API clients
* TLS or other encryption libraries.

[crates.io]: https://crates.io/crates/cotton
[Latest Version]: https://img.shields.io/crates/v/cotton.svg
[Documentation]: https://docs.rs/cotton/badge.svg
[docs.rs]: https://docs.rs/cotton
[License]: https://img.shields.io/crates/l/cotton.svg
