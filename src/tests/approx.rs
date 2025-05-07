// SPDX-License-Identifier: BSD-2-Clause
// See LICENSE file in the project root for full license text.

use crate::{RegApproxParams, RegcompFlags, Regex, RegexecFlags};

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
    assert!(matched_0.unwrap().is_ok());
    assert_eq!(*(matched_0.unwrap().as_ref().unwrap()), "hullo warld");

    let matched_1 = matched[1].as_ref();
    assert!(matched_1.is_some());
    assert!(matched_1.unwrap().is_ok());
    assert_eq!(*(matched_1.unwrap().as_ref().unwrap()), "hullo");

    let matched_2 = matched[2].as_ref();
    assert!(matched_2.is_some());
    assert!(matched_2.unwrap().is_ok());
    assert_eq!(*(matched_2.unwrap().as_ref().unwrap()), "warld");
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
    assert_eq!(matched_0.unwrap().as_ref(), b"hullo warld");

    let matched_1 = matched[1].as_ref();
    assert!(matched_1.is_some());
    assert_eq!(matched_1.unwrap().as_ref(), b"hullo");

    let matched_2 = matched[2].as_ref();
    assert!(matched_2.is_some());
    assert_eq!(matched_2.unwrap().as_ref(), b"warld");
}
