tre-regex
---------

[![CI](https://github.com/Elizafox/tre-regex/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Elizafox/tre-regex/actions/workflows/ci.yml)

Safe API bindings to the [TRE regex engine](https://laurikari.net/tre).

Documentation is available at [docs.rs](https://docs.rs/crate/tre-regex/latest).

Requires Rust 1.85.1 or newer.

```rust
use tre_regex::{Regex, RegcompFlags, RegexecFlags};

fn main() -> tre_regex::Result<()> {
    let regex = Regex::new(
        "^([[:alpha:]]+) ([[:alpha:]]+)$",
        RegcompFlags::EXTENDED | RegcompFlags::ICASE,
    )?;

    assert!(regex.is_match("hello world", RegexecFlags::NONE)?);
    let captures = regex.captures("hello world", 3, RegexecFlags::NONE)?;
    assert_eq!(captures[1], Some("hello"));
    Ok(())
}
```

Approximate matching
====================

Approximate matching allows insertions, deletions, and substitutions with
configurable costs and limits. It requires the `approx` feature, which is
enabled by default.

```rust
use tre_regex::{Regex, RegApproxParams, RegcompFlags, RegexecFlags};

fn main() -> tre_regex::Result<()> {
    let regex = Regex::new("^(hello).*(world)$", RegcompFlags::EXTENDED)?;
    let params = RegApproxParams::new()
        .cost_ins(1)
        .cost_del(1)
        .cost_subst(1)
        .max_cost(2)
        .max_ins(2)
        .max_del(2)
        .max_subst(2)
        .max_err(2);

    let result = regex.regaexec("hullo warld", &params, 3, RegexecFlags::NONE)?;
    assert_eq!(result.cost(), 2);
    assert_eq!(result.get_matches()[0], Some("hullo warld"));
    assert_eq!(result.get_matches()[1], Some("hullo"));
    assert_eq!(result.get_matches()[2], Some("warld"));
    Ok(())
}
```

Wide-character matching
=======================

Wide-character support requires the default `wchar` feature and a dependency on the
[`widestring`](https://crates.io/crates/widestring) crate.

```rust
use tre_regex::{Regex, RegcompFlags, RegexecFlags};
use widestring::widestr;

fn main() -> tre_regex::Result<()> {
    let regex = Regex::new_wide(
        widestr!("^(hello).*(world)$"),
        RegcompFlags::EXTENDED | RegcompFlags::ICASE,
    )?;
    let captures = regex.regwexec(widestr!("hello wide world"), 3, RegexecFlags::NONE)?;

    assert_eq!(captures[1], Some(widestr!("hello")));
    assert_eq!(captures[2], Some(widestr!("world")));
    Ok(())
}
```

Features
========

- `wchar`: enable wide-character regex support. **Enabled by default.**
- `approx`: enable approximate matching support. **Enabled by default.**
- `vendored`: use the vendored copy of TRE with
  [tre-regex-sys](https://crates.io/crates/tre-regex-sys); otherwise use the
  system TRE. **Enabled by default.**

Contributing
============

Contributions are welcome. Please follow the [Code of Conduct](CODE_OF_CONDUCT.md)
and guidelines set out by the [Developer Certificate of Origin](DCO).
