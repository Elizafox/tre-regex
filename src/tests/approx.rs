// SPDX-License-Identifier: BSD-2-Clause
// See LICENSE file in the project root for full license text.

use crate::{
    RegApproxParams, RegcompFlags, Regex, RegexError, RegexecFlags, regaexec, regaexec_bytes, tre,
};

#[test]
fn test_regaexec() {
    let regcomp_flags = RegcompFlags::new()
        .add(RegcompFlags::EXTENDED)
        .add(RegcompFlags::ICASE);
    let regaexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    let regaexec_params = RegApproxParams::new()
        .cost_ins(1)
        .cost_del(1)
        .cost_subst(1)
        .max_cost(2)
        .max_del(2)
        .max_ins(2)
        .max_subst(2)
        .max_err(2);

    let compiled_reg = Regex::new("^(hello).*(world)$", regcomp_flags).expect("Regex::new");
    let result = compiled_reg
        .regaexec(
            "hullo warld",    // String to match against
            &regaexec_params, // Matching parameters
            3,                // Number of matches we want
            regaexec_flags,   // Flags
        )
        .expect("regaexec");

    let matched = result.get_matches();

    let matched_0 = matched[0].as_ref();
    assert!(matched_0.is_some());
    assert_eq!(matched_0.copied(), Some("hullo warld"));

    let matched_1 = matched[1].as_ref();
    assert!(matched_1.is_some());
    assert_eq!(matched_1.copied(), Some("hullo"));

    let matched_2 = matched[2].as_ref();
    assert!(matched_2.is_some());
    assert_eq!(matched_2.copied(), Some("warld"));
}

#[test]
fn test_regaexec_bytes() {
    let regcomp_flags = RegcompFlags::new()
        .add(RegcompFlags::EXTENDED)
        .add(RegcompFlags::ICASE);
    let regaexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    let regaexec_params = RegApproxParams::new()
        .cost_ins(1)
        .cost_del(1)
        .cost_subst(1)
        .max_cost(2)
        .max_del(2)
        .max_ins(2)
        .max_subst(2)
        .max_err(2);

    let compiled_reg = Regex::new_bytes(b"^(hello).*(world)$", regcomp_flags).expect("Regex::new");
    let result = compiled_reg
        .regaexec_bytes(
            b"hullo warld",   // String to match against
            &regaexec_params, // Matching parameters
            3,                // Number of matches we want
            regaexec_flags,   // Flags
        )
        .expect("regaexec");

    let matched = result.get_matches();

    let matched_0 = matched[0].as_ref();
    assert!(matched_0.is_some());
    assert_eq!(*matched_0.unwrap(), b"hullo warld");

    let matched_1 = matched[1].as_ref();
    assert!(matched_1.is_some());
    assert_eq!(*matched_1.unwrap(), b"hullo");

    let matched_2 = matched[2].as_ref();
    assert!(matched_2.is_some());
    assert_eq!(*matched_2.unwrap(), b"warld");
}

#[test]
fn regaexec_bytes_matches_non_utf8() {
    let flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
    let params = RegApproxParams::new()
        .cost_ins(1)
        .cost_del(1)
        .cost_subst(1)
        .max_cost(1)
        .max_del(1)
        .max_ins(1)
        .max_subst(1)
        .max_err(1);
    let regex = Regex::new_bytes(b"\xff", flags).expect("compile non-UTF-8 pattern");
    let result = regex
        .regaexec_bytes(b"\xfe", &params, 1, RegexecFlags::new())
        .expect("approximately match non-UTF-8 data");

    assert_eq!(result.cost(), 1);
    assert_eq!(result.get_matches()[0], Some(b"\xfe".as_slice()));
}

#[test]
fn approximate_accessors_wrappers_none_and_errors_work() {
    let params = RegApproxParams::new()
        .cost_ins(1)
        .cost_del(1)
        .cost_subst(1)
        .max_cost(1)
        .max_del(1)
        .max_ins(1)
        .max_subst(1)
        .max_err(1);
    let regex = Regex::new("^(hello)?world$", RegcompFlags::EXTENDED).expect("compile regex");
    let result =
        regaexec(&regex, "wurld", &params, 2, RegexecFlags::NONE).expect("approximate match");
    assert_eq!(result.cost(), 1);
    assert_eq!(result.num_ins(), 0);
    assert_eq!(result.num_del(), 0);
    assert_eq!(result.num_subst(), 1);
    assert_eq!(result.get_orig_data(), "wurld");
    assert_eq!(result.into_matches(), vec![Some("wurld"), None]);

    let bytes = Regex::new_bytes(b"world", RegcompFlags::EXTENDED).expect("compile bytes");
    assert_eq!(
        regaexec_bytes(&bytes, b"wurld", &params, 1, RegexecFlags::NONE)
            .unwrap()
            .get_matches()[0],
        Some(b"wurld".as_slice())
    );
    let optional_bytes =
        Regex::new_bytes(b"^(hello)?world$", RegcompFlags::EXTENDED).expect("compile bytes");
    assert_eq!(
        optional_bytes
            .regaexec_bytes(b"wurld", &params, 2, RegexecFlags::NONE)
            .unwrap()
            .get_matches()[1],
        None
    );
    let byte_no_match = bytes
        .regaexec_bytes(b"unrelated", &RegApproxParams::new(), 1, RegexecFlags::NONE)
        .expect_err("approximate byte non-match");
    assert!(matches!(
        byte_no_match,
        RegexError::Tre {
            code: tre::reg_errcode_t::REG_NOMATCH,
            ..
        }
    ));

    let no_match = regex
        .regaexec("unrelated", &RegApproxParams::new(), 1, RegexecFlags::NONE)
        .expect_err("approximate non-match");
    assert!(matches!(
        no_match,
        RegexError::Tre {
            code: tre::reg_errcode_t::REG_NOMATCH,
            ..
        }
    ));

    let vacant = Regex(None);
    assert!(
        vacant
            .regaexec("x", &params, 1, RegexecFlags::NONE)
            .is_err()
    );
    assert!(
        vacant
            .regaexec_bytes(b"x", &params, 1, RegexecFlags::NONE)
            .is_err()
    );
}

#[test]
fn approximate_string_match_rejects_invalid_utf8_boundaries() {
    let regex = Regex::new_bytes(b"\xa9", RegcompFlags::EXTENDED | RegcompFlags::USEBYTES)
        .expect("compile byte suffix");
    let params = RegApproxParams::new();
    let error = regex
        .regaexec("é", &params, 1, RegexecFlags::NONE)
        .expect_err("offset bisects UTF-8 character");
    assert!(matches!(error, RegexError::InvalidUtf8Boundary));
}
