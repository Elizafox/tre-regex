// SPDX-License-Identifier: BSD-2-Clause
// See LICENSE file in the project root for full license text.

use crate::{ErrorKind, RegcompFlags, regcomp, tre};

#[test]
fn regerror_works() {
    match regcomp("[a", RegcompFlags::NONE) {
        Ok(_) => panic!("regcomp"),
        Err(e) => {
            assert_eq!(e.kind, ErrorKind::Tre(tre::reg_errcode_t::REG_EBRACK));
            assert_eq!(e.error, "Missing ']'");
        }
    }
}
