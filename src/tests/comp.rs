use crate::{regcomp, regcomp_bytes, RegcompFlags};

#[test]
fn regcomp_flags_works() {
    let regcomp_flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
    assert_eq!(regcomp_flags.get(), RegcompFlags::EXTENDED);

    let regcomp_flags = regcomp_flags.add(RegcompFlags::ICASE);
    assert_eq!(
        regcomp_flags.get(),
        RegcompFlags::EXTENDED | RegcompFlags::ICASE
    );

    let regcomp_flags = regcomp_flags.remove(RegcompFlags::EXTENDED);
    assert_eq!(regcomp_flags.get(), RegcompFlags::ICASE);
}

#[test]
fn regcomp_works() {
    assert!(
        regcomp("[A-Za-z0-9]*", RegcompFlags::new().add(RegcompFlags::BASIC)).is_ok(),
        "regcomp"
    );

    assert!(
        regcomp(
            "[[:alpha:]]*",
            RegcompFlags::new()
                .add(RegcompFlags::EXTENDED)
                .add(RegcompFlags::ICASE)
        )
        .is_ok(),
        "regcomp"
    );
}

#[test]
fn regcomp_bytes_works() {
    assert!(
        regcomp_bytes(
            b"[A-Za-z0-9]*",
            RegcompFlags::new().add(RegcompFlags::BASIC)
        )
        .is_ok(),
        "regcomp"
    );

    assert!(
        regcomp_bytes(
            b"[[:alpha:]]*",
            RegcompFlags::new()
                .add(RegcompFlags::EXTENDED)
                .add(RegcompFlags::ICASE)
        )
        .is_ok(),
        "regcomp"
    );
}
