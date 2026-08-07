// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

use crate::{RegcompFlags, RegexecFlags};

#[test]
fn flag_helpers_and_operators_work() {
    let unknown = RegcompFlags::from_bits_retain(1 << 30);
    assert_eq!(unknown.bits(), 1 << 30);
    assert!(RegcompFlags::new().is_empty());

    let mut compile = RegcompFlags::EXTENDED;
    compile |= RegcompFlags::ICASE;
    assert!(compile.contains(RegcompFlags::EXTENDED | RegcompFlags::ICASE));
    assert!(compile.intersects(RegcompFlags::ICASE));
    compile &= !RegcompFlags::EXTENDED;
    assert_eq!(compile, RegcompFlags::ICASE);
    assert_eq!(compile & RegcompFlags::EXTENDED, RegcompFlags::NONE);

    let mut execute = RegexecFlags::NOTBOL;
    execute |= RegexecFlags::NOTEOL;
    assert!(execute.intersects(RegexecFlags::NOTBOL));
    execute &= !RegexecFlags::NOTBOL;
    assert_eq!(execute, RegexecFlags::NOTEOL);
    assert_eq!(execute.remove(RegexecFlags::NOTEOL), RegexecFlags::NONE);
    assert_eq!(
        RegexecFlags::from_bits_retain(execute.bits()),
        RegexecFlags::NOTEOL
    );
}
