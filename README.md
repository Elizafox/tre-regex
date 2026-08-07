tre-regex
---------

[![CI](https://github.com/Elizafox/tre-regex/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Elizafox/tre-regex/actions/workflows/ci.yml)

Safe API bindings to the [TRE regex engine](https://laurikari.net/tre).

Documentation is available at [docs.rs](https://docs.rs/crate/tre-regex/latest).

Requires Rust 1.85.1 or newer.

Features
========

- `wchar`: enable wide-character regex support. **Enabled by default.**
- `approx`: enable approximate matching support. **Enabled by default.**
- `vendored`: use the vendored copy of TRE with [tre-regex-sys](https://crates.io/crates/tre-regex-sys); otherwise use the system TRE. **Enabled by default.**
