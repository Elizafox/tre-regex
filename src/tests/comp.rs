// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

use crate::{RegcompFlags, RegexError, regcomp, regcomp_bytes, tre};

#[test]
fn regcomp_flags_works() {
    let regcomp_flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
    assert_eq!(regcomp_flags, RegcompFlags::EXTENDED);

    let regcomp_flags = regcomp_flags.add(RegcompFlags::ICASE);
    assert_eq!(regcomp_flags, RegcompFlags::EXTENDED | RegcompFlags::ICASE);

    let regcomp_flags = regcomp_flags.remove(RegcompFlags::EXTENDED);
    assert_eq!(regcomp_flags, RegcompFlags::ICASE);
}

#[test]
fn regcomp_bytes_reports_compile_errors() {
    let error = regcomp_bytes(b"[", RegcompFlags::EXTENDED).expect_err("invalid byte regex");
    assert!(matches!(
        error,
        RegexError::Tre {
            code: tre::reg_errcode_t::REG_EBRACK,
            ..
        }
    ));
}

#[test]
fn regcomp_works() {
    assert!(
        regcomp("[A-Za-z0-9]*", RegcompFlags::new().add(RegcompFlags::BASIC)).is_ok(),
        "regcomp"
    );

    assert!(
        regcomp(
            "[[:alpha:]]*",
            RegcompFlags::new()
                .add(RegcompFlags::EXTENDED)
                .add(RegcompFlags::ICASE)
        )
        .is_ok(),
        "regcomp"
    );
}

#[test]
fn regcomp_bytes_works() {
    assert!(
        regcomp_bytes(
            b"[A-Za-z0-9]*",
            RegcompFlags::new().add(RegcompFlags::BASIC)
        )
        .is_ok(),
        "regcomp"
    );

    assert!(
        regcomp_bytes(
            b"[[:alpha:]]*",
            RegcompFlags::new()
                .add(RegcompFlags::EXTENDED)
                .add(RegcompFlags::ICASE)
        )
        .is_ok(),
        "regcomp"
    );

    assert!(
        regcomp_bytes(b"a\0b", RegcompFlags::new().add(RegcompFlags::EXTENDED)).is_ok(),
        "regcomp_bytes accepts embedded NUL bytes"
    );

    assert!(
        regcomp_bytes(b"\xff", RegcompFlags::new().add(RegcompFlags::EXTENDED)).is_ok(),
        "regcomp_bytes accepts non-UTF-8 bytes"
    );
}
