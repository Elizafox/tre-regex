// SPDX-License-Identifier: BSD-2-Clause
// See LICENSE file in the project root for full license text.

use crate::{RegcompFlags, RegexecFlags, regcomp, regcomp_bytes, regexec, regexec_bytes};

#[test]
fn regexec_flags_works() {
    let regexec_flags = RegexecFlags::new().add(RegexecFlags::NOTBOL);
    assert_eq!(regexec_flags, RegexecFlags::NOTBOL);

    let regexec_flags = regexec_flags.add(RegexecFlags::NOTEOL);
    assert_eq!(regexec_flags, RegexecFlags::NOTBOL | RegexecFlags::NOTEOL);

    let regexec_flags = regexec_flags.remove(RegexecFlags::NOTBOL);
    assert_eq!(regexec_flags, RegexecFlags::NOTEOL);
    assert!(regexec_flags.contains(RegexecFlags::NOTEOL));
    assert!(!regexec_flags.is_empty());
    assert_eq!(regexec_flags.bits(), crate::tre::REG_NOTEOL);
}

#[test]
fn is_match_distinguishes_a_non_match_from_an_error() {
    let regex = regcomp("hello", RegcompFlags::EXTENDED).expect("compile regex");
    assert!(regex.is_match("hello", RegexecFlags::NONE).unwrap());
    assert!(!regex.is_match("goodbye", RegexecFlags::NONE).unwrap());
}

#[test]
fn regexec_works() {
    let regcomp_flags = RegcompFlags::new().add(RegcompFlags::BASIC);
    let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    let Ok(compiled_reg) = regcomp("[A-Za-z0-9]*", regcomp_flags) else {
        panic!("regcomp");
    };
    let Ok(result) = regexec(&compiled_reg, "hello", 2, regexec_flags) else {
        panic!("regexec");
    };
    assert!(result[0].is_some());
    assert_eq!(result[0], Some("hello"));
    assert!(result[1].is_none());
}

#[test]
fn regexec_bytes_works() {
    let regcomp_flags = RegcompFlags::new().add(RegcompFlags::BASIC);
    let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    let Ok(compiled_reg) = regcomp_bytes(b"[A-Za-z0-9]*", regcomp_flags) else {
        panic!("regcomp");
    };
    let Ok(result) = regexec_bytes(&compiled_reg, b"hello", 2, regexec_flags) else {
        panic!("regexec_bytes");
    };
    assert!(result[0].is_some());
    assert_eq!(result[0], Some(b"hello".as_slice()));
    assert!(result[1].as_ref().is_none());
}

#[test]
fn regexec_bytes_matches_embedded_nul_and_non_utf8() {
    let flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
    let exec_flags = RegexecFlags::new();

    let nul_regex = regcomp_bytes(b"a\0b", flags).expect("compile embedded NUL pattern");
    let nul_matches =
        regexec_bytes(&nul_regex, b"xxa\0byy", 1, exec_flags).expect("match embedded NUL data");
    assert_eq!(nul_matches[0], Some(b"a\0b".as_slice()));

    let byte_regex = regcomp_bytes(b"\xff", flags).expect("compile non-UTF-8 pattern");
    let byte_matches =
        regexec_bytes(&byte_regex, b"x\xffy", 1, exec_flags).expect("match non-UTF-8 data");
    assert_eq!(byte_matches[0], Some(b"\xff".as_slice()));
}

#[test]
fn regex_multibyte_works() {
    let regcomp_flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
    let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    let Ok(compiled_reg) = regcomp(".*(エリザベス).*", regcomp_flags) else {
        panic!("regcomp");
    };
    let Ok(result) = regexec(&compiled_reg, "私の名前はエリザベスです", 2, regexec_flags)
    else {
        panic!("regexec");
    };
    assert!(result[0].is_some());
    assert_eq!(result[0], Some("私の名前はエリザベスです"));
    assert!(result[1].is_some());
    assert_eq!(result[1], Some("エリザベス"));
}
