# TSL UMD protocols 🦀 📹
[docs](https://docs.rs/tsl-umd) | [crate](https://crates.io/crate/tsl-umd)

This is a Rust implementation of the TSL UMD family of tally protocols.

## Features
- [x] decoding
  - [x] v3.1
  - [ ] v4.0 (soon)
  - [ ] v5.0 (soon)
- [ ] encoding - easily construct packets (soon)
- [x] `no_std` - runs on anything
- [x] Zero copy (more or less) - fields are extracted from the buffer when you need them.
  If you don't need them, there's no overhead.
