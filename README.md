tre-regex
---------
Safe API bindings to the [TRE regex engine](https://laurikari.net/tre).

Documentation is available at [docs.rs](https://docs.rs/crate/tre-regex/latest).

Should work on Rust 1.70.0 and up. Please report it if you discover otherwise.

Features
========
* `wchar`: enable wchar support (not yet supported by the bindings, but will be enabled in [tre-regex-sys](https://crates.io/crates/tre-regex-sys)). **Enabled by default.**
* `approx`: enable approximate matching support. **Enabled by default.**
* `vendored`: use the vendored copy of TRE with [tre-regex-sys](https://crates.io/crates/tre-regex-sys); otherwise use the system TRE. **Enabled by default.**
