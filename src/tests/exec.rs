// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

use crate::{
    RegcompFlags, Regex, RegexError, RegexecFlags, match_offset, regcomp, regcomp_bytes, regexec,
    regexec_bytes, tre,
};

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
    assert_eq!(regexec_flags.bits(), tre::REG_NOTEOL);
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

#[test]
fn byte_match_helpers_and_errors_work() {
    let regex = regcomp_bytes(b"(hello)?world", RegcompFlags::EXTENDED).expect("compile bytes");
    assert!(regex.is_match_bytes(b"world", RegexecFlags::NONE).unwrap());
    assert!(
        !regex
            .is_match_bytes(b"goodbye", RegexecFlags::NONE)
            .unwrap()
    );
    assert_eq!(
        regex
            .captures_bytes(b"world", 2, RegexecFlags::NONE)
            .unwrap(),
        vec![Some(b"world".as_slice()), None]
    );

    let vacant = Regex(None);
    assert!(matches!(
        vacant
            .is_match("x", RegexecFlags::NONE)
            .expect_err("vacant match"),
        RegexError::VacantRegex
    ));
    assert!(vacant.regexec("x", 1, RegexecFlags::NONE).is_err());
    assert!(vacant.regexec_bytes(b"x", 1, RegexecFlags::NONE).is_err());
    assert!(vacant.is_match_bytes(b"x", RegexecFlags::NONE).is_err());
}

#[test]
fn string_match_rejects_byte_offsets_inside_utf8_characters() {
    let regex = regcomp_bytes(b"\xa9", RegcompFlags::EXTENDED | RegcompFlags::USEBYTES)
        .expect("compile byte suffix");
    let error = regex
        .captures("é", 1, RegexecFlags::NONE)
        .expect_err("offset bisects UTF-8 character");
    assert!(matches!(error, RegexError::InvalidUtf8Boundary));
}

#[test]
fn match_offset_rejects_negative_values() {
    use std::error::Error;

    let error = match_offset(-1).expect_err("negative offset");
    assert!(error.source().is_some());
    assert!(matches!(error, RegexError::InvalidMatchOffset { .. }));
}

#[test]
fn captures_and_execution_error_paths_work() {
    let regex = regcomp("(hello)?world", RegcompFlags::EXTENDED).expect("compile regex");
    assert_eq!(
        regex.captures("world", 2, RegexecFlags::NONE).unwrap(),
        vec![Some("world"), None]
    );
    let error = regexec(&regex, "goodbye", 1, RegexecFlags::NONE).expect_err("non-match");
    assert!(matches!(
        error,
        RegexError::Tre {
            code: tre::reg_errcode_t::REG_NOMATCH,
            ..
        }
    ));
}
