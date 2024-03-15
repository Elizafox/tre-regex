use crate::{regcomp, regexec, regexec_bytes, RegcompFlags, RegexecFlags};

#[test]
fn regexec_flags_works() {
    let regexec_flags = RegexecFlags::new().add(RegexecFlags::NOTBOL);
    assert_eq!(regexec_flags.get(), RegexecFlags::NOTBOL);

    let regexec_flags = regexec_flags.add(RegexecFlags::NOTEOL);
    assert_eq!(
        regexec_flags.get(),
        RegexecFlags::NOTBOL | RegexecFlags::NOTEOL
    );

    let regexec_flags = regexec_flags.remove(RegexecFlags::NOTBOL);
    assert_eq!(regexec_flags.get(), RegexecFlags::NOTEOL);
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
    assert!(result[0].as_ref().unwrap().is_ok());
    assert_eq!(*result[0].as_ref().unwrap().as_ref().unwrap(), "hello");
    assert!(result[1].is_none());
}

#[test]
fn regexec_bytes_works() {
    let regcomp_flags = RegcompFlags::new().add(RegcompFlags::BASIC);
    let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    let Ok(compiled_reg) = regcomp("[A-Za-z0-9]*", regcomp_flags) else {
        panic!("regcomp");
    };
    let Ok(result) = regexec_bytes(&compiled_reg, b"hello", 2, regexec_flags) else {
        panic!("regexec_bytes");
    };
    assert!(result[0].is_some());
    assert_eq!(result[0].as_ref().unwrap().as_ref(), b"hello");
    assert!(result[1].as_ref().is_none());
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
    assert!(result[0].as_ref().unwrap().is_ok());
    assert_eq!(
        *result[0].as_ref().unwrap().as_ref().unwrap(),
        "私の名前はエリザベスです"
    );
    assert!(result[1].is_some());
    assert!(result[1].as_ref().unwrap().is_ok());
    assert_eq!(*result[1].as_ref().unwrap().as_ref().unwrap(), "エリザベス");
}
