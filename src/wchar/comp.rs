// SPDX-License-Identifier: BSD-2-Clause
// See LICENSE file in the project root for full license text.

use std::mem;

use widestring::WideStr;

use crate::{
    err::{regerror, Result},
    flags::RegcompFlags,
    tre, Regex,
};

impl Regex {
    /// Compiles a regex contained in a [`WideStr`] and wraps it in a `Regex` object.
    ///
    /// # Arguments
    /// * `reg`: regular expression to compile, as a [`WideStr`] .
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
    /// use widestring::widestr;
    ///
    /// let regcomp_flags = RegcompFlags::new().add(RegcompFlags::BASIC);
    /// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    ///
    /// let compiled_reg = Regex::new_wide(widestr!("[A-Za-z0-9]*"), regcomp_flags)?;
    /// let matches = compiled_reg.regwexec(widestr!("hello"), 1, regexec_flags)?;
    ///
    /// for (i, matched) in matches.into_iter().enumerate() {
    ///     match matched {
    ///         Some(substr) => println!("Match {i}: '{}'", substr.display()),
    ///         None => println!("Match {i}: <None>"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`RegexError`]: crate::RegexError
    pub fn new_wide(reg: &WideStr, flags: RegcompFlags) -> Result<Self> {
        let mut unwrapped_compiled_reg = mem::MaybeUninit::<tre::regex_t>::uninit();

        // SAFETY: unwrapped_compiled_reg is being initalised. reg is immutably passed and is not
        // modified by the caller. Wrapping is also impossible.
        #[allow(clippy::cast_possible_wrap)]
        let result = unsafe {
            tre::tre_regwncomp(
                unwrapped_compiled_reg.as_mut_ptr(),
                reg.as_ptr().cast(),
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

/// Compiles a regex that is in the form of a [`WideStr`].
///
/// This is a thin wrapper around [`Regex::new_wide`].
///
/// # Arguments
/// * `reg`: regular expression to compile, as a [`WideStr`].
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
/// # use tre_regex::Result;
///
/// # fn main() -> Result<()> {
/// use tre_regex::{RegcompFlags, RegexecFlags, regwcomp, regwexec};
/// use widestring::widestr;
///
/// let regcomp_flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
/// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
///
/// let compiled_reg = regwcomp(widestr!("[[:digit:]]*"), regcomp_flags)?;
/// let matches = regwexec(&compiled_reg, widestr!("01234567890"), 1, regexec_flags)?;
///
/// for (i, matched) in matches.into_iter().enumerate() {
///     match matched {
///         Some(substr) => println!("Match {i}: '{}'", substr.display()),
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
pub fn regwcomp(reg: &WideStr, flags: RegcompFlags) -> Result<Regex> {
    Regex::new_wide(reg, flags)
}
