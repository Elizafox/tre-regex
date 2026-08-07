// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

use widestring::widestr;

use crate::{RegcompFlags, Regex, RegexError, RegexecFlags, regwcomp, regwexec, tre};

#[cfg(feature = "approx")]
use crate::{RegApproxParams, regawexec};

#[test]
fn wide_compile_and_execute_cover_success_none_and_errors() {
    let flags = RegcompFlags::EXTENDED;
    let regex = regwcomp(widestr!("^(hello)? world$"), flags).expect("compile wide regex");
    let matches =
        regwexec(&regex, widestr!(" world"), 2, RegexecFlags::NONE).expect("execute wide regex");
    assert_eq!(matches[0], Some(widestr!(" world")));
    assert_eq!(matches[1], None);

    let error = Regex::new_wide(widestr!("["), flags).expect_err("invalid wide regex");
    assert!(matches!(
        error,
        RegexError::Tre {
            code: tre::reg_errcode_t::REG_EBRACK,
            ..
        }
    ));

    let no_match = regex
        .regwexec(widestr!("goodbye"), 1, RegexecFlags::NONE)
        .expect_err("wide non-match");
    assert!(matches!(
        no_match,
        RegexError::Tre {
            code: tre::reg_errcode_t::REG_NOMATCH,
            ..
        }
    ));
}

#[test]
fn vacant_wide_regex_is_rejected() {
    let regex = Regex(None);
    assert!(Regex::new_wide(widestr!("x"), RegcompFlags::EXTENDED).is_ok());
    assert!(
        regex
            .regwexec(widestr!("x"), 1, RegexecFlags::NONE)
            .is_err()
    );
}

#[cfg(feature = "approx")]
#[test]
fn approximate_wide_execute_covers_metadata_none_and_errors() {
    let params = RegApproxParams::new()
        .cost_ins(1)
        .cost_del(1)
        .cost_subst(1)
        .max_cost(1)
        .max_ins(1)
        .max_del(1)
        .max_subst(1)
        .max_err(1);
    let regex = Regex::new_wide(widestr!("^(hello)? world$"), RegcompFlags::EXTENDED)
        .expect("compile wide regex");
    let result = regawexec(&regex, widestr!(" warld"), &params, 2, RegexecFlags::NONE)
        .expect("approximately execute wide regex");
    assert_eq!(result.cost(), 1);
    assert_eq!(result.get_orig_data(), widestr!(" warld"));
    assert_eq!(result.get_matches()[1], None);

    let no_match = regex
        .regawexec(
            widestr!("unrelated"),
            &RegApproxParams::new(),
            1,
            RegexecFlags::NONE,
        )
        .expect_err("wide approximate non-match");
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
            .regawexec(widestr!("x"), &params, 1, RegexecFlags::NONE)
            .is_err()
    );
}
