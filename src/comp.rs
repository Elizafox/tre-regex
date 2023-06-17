use std::ffi::c_char;
use std::mem;

use crate::{
    err::{regerror, Result},
    flags::RegcompFlags,
    tre, Regex,
};

impl Regex {
    /// Compiles a regex and wraps it in a `Regex` object.
    ///
    /// # Arguments
    /// * `reg`: regular expression to compile, as a string.
    /// * `flags`: [`RegcompFlags`] to pass to the function.
    ///
    /// # Returns
    /// An opaque [`Regex`] object will be returned. It will be freed automatically when dropped.
    ///
    /// # Errors
    /// Will return a [`RegexError`] upon failure.
    ///
    /// # Examples
    /// ```
    /// # use tre_regex::Result;
    /// # fn main() -> Result<()> {
    /// use tre_regex::{RegcompFlags, RegexecFlags, Regex};
    ///
    /// let regcomp_flags = RegcompFlags::new().add(RegcompFlags::BASIC);
    /// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    ///
    /// let compiled_reg = Regex::new("[A-Za-z0-9]*", regcomp_flags)?;
    /// let matches = compiled_reg.regexec("hello", 1, regexec_flags)?;
    ///
    /// for (i, matched) in matches.into_iter().enumerate() {
    ///     match matched {
    ///         Some(res) => {
    ///             match res {
    ///                 Ok(substr) => println!("Match {i}: '{}'", substr),
    ///                 Err(e) => println!("Match {i}: <Error: {e}>"),
    ///             }
    ///         },
    ///         None => println!("Match {i}: <None>"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`RegexError`]: crate::RegexError
    pub fn new(reg: &str, flags: RegcompFlags) -> Result<Self> {
        Self::new_bytes(reg.as_bytes(), flags)
    }

    /// Compiles a regex contained in a `u8` slice and wraps it in a `Regex` object.
    ///
    /// # Arguments
    /// * `reg`: regular expression to compile, as a string.
    /// * `flags`: [`RegcompFlags`] to pass to the function.
    ///
    /// # Returns
    /// An opaque [`Regex`] object will be returned. It will be freed automatically when dropped.
    ///
    /// # Errors
    /// Will return a [`RegexError`] upon failure.
    ///
    /// # Examples
    /// ```
    /// # use tre_regex::Result;
    /// # fn main() -> Result<()> {
    /// use tre_regex::{RegcompFlags, RegexecFlags, Regex};
    ///
    /// let regcomp_flags = RegcompFlags::new().add(RegcompFlags::BASIC);
    /// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    ///
    /// let compiled_reg = Regex::new_bytes(b"[A-Za-z0-9]*", regcomp_flags)?;
    /// let matches = compiled_reg.regexec("hello", 1, regexec_flags)?;
    ///
    /// for (i, matched) in matches.into_iter().enumerate() {
    ///     match matched {
    ///         Some(res) => {
    ///             match res {
    ///                 Ok(substr) => println!("Match {i}: '{}'", substr),
    ///                 Err(e) => println!("Match {i}: <Error: {e}>"),
    ///             }
    ///         },
    ///         None => println!("Match {i}: <None>"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`RegexError`]: crate::RegexError
    pub fn new_bytes(reg: &[u8], flags: RegcompFlags) -> Result<Self> {
        let mut unwrapped_compiled_reg = mem::MaybeUninit::<tre::regex_t>::uninit();

        // SAFETY: unwrapped_compiled_reg is being initalised. reg is immutably passed and is not
        // modified by the caller. Wrapping is also impossible.
        #[allow(clippy::cast_possible_wrap)]
        let result = unsafe {
            tre::tre_regncomp(
                unwrapped_compiled_reg.as_mut_ptr(),
                reg.as_ptr().cast::<c_char>(),
                reg.len(),
                flags.get(),
            )
        };

        // SAFETY: tre::tre_regcomp fully initalises compiled_reg
        let compiled_reg = Self(Some(unsafe { unwrapped_compiled_reg.assume_init() }));
        if result != 0 {
            return Err(regerror(&compiled_reg, result));
        }

        Ok(compiled_reg)
    }
}

/// Compiles a regex.
///
/// This is a thin wrapper around [`Regex::new`].
///
/// # Arguments
/// * `reg`: regular expression to compile, as a string.
/// * `flags`: [`RegcompFlags`] to pass to the function.
///
/// # Returns
/// An opaque [`Regex`] object will be returned.
///
/// # Errors
/// Will return a [`RegexError`] upon failure.
///
/// # Examples
/// ```
/// use tre_regex::{RegcompFlags, RegexecFlags, regcomp, regexec};
/// # use tre_regex::Result;
///
/// # fn main() -> Result<()> {
/// let regcomp_flags = RegcompFlags::new().add(RegcompFlags::BASIC);
/// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
/// let compiled_reg = regcomp("[A-Za-z0-9]*", regcomp_flags)?;
/// let matches = regexec(&compiled_reg, "hello", 1, regexec_flags)?;
/// for (i, matched) in matches.into_iter().enumerate() {
///     match matched {
///         Some(substr) => {
///             match substr {
///                 Ok(substr) => println!("Match {i}: '{}'", substr),
///                 Err(e) => println!("Match {i}: <Error: {e}>"),
///             }
///         },
///         None => println!("Match {i}: <None>"),
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// [`RegcompFlags`]: crate::RegcompFlags
/// [`RegexError`]: crate::RegexError
#[inline]
pub fn regcomp(reg: &str, flags: RegcompFlags) -> Result<Regex> {
    Regex::new(reg, flags)
}

/// Compiles a regex that is in the form of bytes.
///
/// This is a thin wrapper around [`Regex::new_bytes`].
///
/// # Arguments
/// * `reg`: regular expression to compile, as a string.
/// * `flags`: [`RegcompFlags`] to pass to the function.
///
/// # Returns
/// An opaque [`Regex`] object will be returned.
///
/// # Errors
/// Will return a [`RegexError`] upon failure.
///
/// # Examples
/// ```
/// use tre_regex::{RegcompFlags, RegexecFlags, regcomp_bytes, regexec_bytes};
/// # use tre_regex::Result;
///
/// # fn main() -> Result<()> {
/// let regcomp_flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
/// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
/// let compiled_reg = regcomp_bytes(b"[[:digit:]]*", regcomp_flags)?;
/// let matches = regexec_bytes(&compiled_reg, b"01234567890", 1, regexec_flags)?;
/// for (i, matched) in matches.into_iter().enumerate() {
///     match matched {
///         Some(substr) => println!("Match {i}: '{}'",
///             std::str::from_utf8(substr.as_ref()).unwrap()
///         ),
///         None => println!("Match {i}: <None>"),
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// [`RegcompFlags`]: crate::RegcompFlags
/// [`RegexError`]: crate::RegexError
#[inline]
pub fn regcomp_bytes(reg: &[u8], flags: RegcompFlags) -> Result<Regex> {
    Regex::new_bytes(reg, flags)
}
