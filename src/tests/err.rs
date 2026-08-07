// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

use crate::{RegcompFlags, Regex, RegexError, regcomp, regerror, tre};

#[test]
fn regerror_works() {
    match regcomp("[a", RegcompFlags::NONE) {
        Ok(_) => panic!("regcomp"),
        Err(e) => {
            assert!(matches!(
                e,
                RegexError::Tre {
                    code: tre::reg_errcode_t::REG_EBRACK,
                    ref message,
                } if message == "Missing ']'"
            ));
        }
    }
}

#[test]
fn binding_error_display_and_vacant_regerror_work() {
    let vacant = Regex(None);
    let error = regerror(&vacant, 1);
    assert!(matches!(error, RegexError::VacantRegex));
    assert_eq!(error.to_string(), "attempted to use a vacant Regex object");
}
