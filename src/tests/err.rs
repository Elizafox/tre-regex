use crate::{regcomp, tre, ErrorKind, RegcompFlags, RegexecFlags};

#[test]
fn regerror_works() {
    match regcomp("[a", RegcompFlags::new().add(RegexecFlags::NONE)) {
        Ok(_) => panic!("regcomp"),
        Err(e) => {
            assert_eq!(e.kind, ErrorKind::Tre(tre::reg_errcode_t::REG_EBRACK));
            assert_eq!(e.error, "Missing ']'");
        }
    }
}
