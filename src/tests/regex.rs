// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

use crate::{RegcompFlags, Regex, tre};

#[test]
fn raw_regex_access_and_ownership_transfer_work() {
    let mut regex = Regex::new("hello", RegcompFlags::EXTENDED).expect("compile regex");
    assert!(regex.as_raw().is_some());
    // SAFETY: the test only checks the reference and leaves the TRE object unchanged.
    assert!(unsafe { regex.as_raw_mut() }.is_some());

    let raw = regex.into_raw().expect("take raw regex");
    // SAFETY: raw came directly from Regex::into_raw and ownership is transferred to wrapped.
    let wrapped = unsafe { Regex::new_from(raw) };
    assert!(wrapped.as_raw().is_some());
    drop(wrapped);

    let vacant = Regex(None);
    assert!(vacant.as_raw().is_none());
    drop(vacant);
}

#[test]
fn raw_regex_can_be_freed_by_the_caller() {
    let regex = Regex::new("world", RegcompFlags::EXTENDED).expect("compile regex");
    let mut raw = regex.into_raw().expect("take raw regex");
    // SAFETY: into_raw transferred ownership of this initialized object to the test.
    unsafe { tre::tre_regfree(&raw mut raw) };
}
