// SPDX-License-Identifier: BSD-2-Clause
// See LICENSE file in the project root for full license text.

use std::borrow::Cow;

use crate::{
    Regex,
    err::{BindingErrorCode, ErrorKind, RegexError, Result},
    flags::RegexecFlags,
    tre,
};

pub type RegMatchStr<'a> = Vec<Option<Result<Cow<'a, str>>>>;
pub type RegMatchBytes<'a> = Vec<Option<Cow<'a, [u8]>>>;

/// Converts a nonnegative TRE match offset into a Rust slice offset.
///
/// # Errors
/// Returns a [`RegexError`] if the offset cannot be represented by [`usize`].
pub fn match_offset(offset: tre::regoff_t) -> Result<usize> {
    usize::try_from(offset).map_err(|error| {
        RegexError::new(
            ErrorKind::Binding(BindingErrorCode::INVALID_MATCH_OFFSET),
            &format!("Invalid match offset: {error}"),
        )
    })
}

impl Regex {
    /// Performs a regex search on the passed string, returning `nmatches` results.
    ///
    /// Non-matching subexpressions or patterns will return `None` in the results.
    ///
    /// # Arguments
    /// * `string`: string to match against `compiled_reg`
    /// * `nmatches`: number of matches to return
    /// * `flags`: [`RegexecFlags`] to pass to [`tre_regnexec`](tre_regex_sys::tre_regnexec).
    ///
    /// # Returns
    /// If no error was found, a [`Vec`] of [`Option`]s will be returned.
    ///
    /// If a given match index is empty, The `Option` will be `None`. Otherwise, [`Result`]s will be
    /// returned, containing either errors or substrings of the matches. Errors may be returned due to
    /// decoding problems, such as split codepoints.
    ///
    /// # Errors
    /// If an error is encountered during matching, it returns a [`RegexError`]. Match results may also
    /// return errors, if decoding into UTF-8 was unsuccessful for whatever reason.
    ///
    /// # Caveats
    /// Unless copied, the match results must live at least as long as `string`. This is because they are
    /// slices into `string` under the hood, for efficiency.
    ///
    /// # Examples
    /// ```
    /// # use tre_regex::Result;
    /// # fn main() -> Result<()> {
    /// use tre_regex::{RegcompFlags, RegexecFlags, Regex};
    ///
    /// let regcomp_flags = RegcompFlags::new()
    ///     .add(RegcompFlags::EXTENDED)
    ///     .add(RegcompFlags::ICASE)
    ///     .add(RegcompFlags::UNGREEDY);
    /// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    ///
    /// let compiled_reg = Regex::new_bytes(b"^(hello).*(world)$", regcomp_flags)?;
    /// let matches = compiled_reg.regexec("hello world", 3, regexec_flags)?;
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
    #[inline]
    pub fn regexec<'a>(
        &self,
        string: &'a str,
        nmatches: usize,
        flags: RegexecFlags,
    ) -> Result<RegMatchStr<'a>> {
        let Some(compiled_reg_obj) = self.get() else {
            return Err(RegexError::new(
                ErrorKind::Binding(BindingErrorCode::REGEX_VACANT),
                "Attempted to unwrap a vacant Regex object",
            ));
        };
        let data = string.as_bytes();
        let mut match_vec = vec![tre::regmatch_t::default(); nmatches];

        // SAFETY: the regex is initialised, data is valid for its supplied length, and match_vec
        // contains nmatches writable entries.
        let result = unsafe {
            tre::tre_regnexec(
                compiled_reg_obj,
                data.as_ptr().cast(),
                data.len(),
                nmatches,
                match_vec.as_mut_ptr(),
                flags.get(),
            )
        };
        if result != 0 {
            return Err(self.regerror(result));
        }

        let mut result: Vec<Option<Result<Cow<'a, str>>>> = Vec::with_capacity(nmatches);
        for pmatch in match_vec {
            if pmatch.rm_so < 0 || pmatch.rm_eo < 0 {
                result.push(None);
                continue;
            }

            let start_offset = match_offset(pmatch.rm_so)?;
            let end_offset = match_offset(pmatch.rm_eo)?;
            let pmatch = &data[start_offset..end_offset];

            result.push(Some(match std::str::from_utf8(pmatch) {
                Ok(s) => Ok(s.into()),
                Err(e) => Err(RegexError::new(
                    ErrorKind::Binding(BindingErrorCode::ENCODING),
                    &format!("UTF-8 encoding error: {e}"),
                )),
            }));
        }

        Ok(result)
    }

    /// Performs a regex search on the passed bytes, returning `nmatches` results.
    ///
    /// This function should only be used if you need to match raw bytes, or bytes which may not be
    /// UTF-8 compliant. Otherwise, [`regexec`] is recommended instead.
    ///
    /// # Arguments
    /// * `data`: [`u8`] slice to match against `compiled_reg`
    /// * `nmatches`: number of matches to return
    /// * `flags`: [`RegexecFlags`] to pass to [`tre_regnexecb`](tre_regex_sys::tre_regnexecb).
    ///
    /// # Returns
    /// If no error was found, a [`Vec`] of [`Option`]s will be returned.
    ///
    /// If a given match index is empty, The `Option` will be `None`. Otherwise, [`u8`] slices will be
    /// returned.
    ///
    /// # Errors
    /// If an error is encountered during matching, it returns a [`RegexError`].
    ///
    /// # Caveats
    /// Unless copied, the match results must live at least as long as `data`. This is because they are
    /// slices into `data` under the hood, for efficiency.
    ///
    /// # Examples
    /// ```
    /// # use tre_regex::Result;
    /// # fn main() -> Result<()> {
    /// use tre_regex::{RegcompFlags, RegexecFlags, Regex};
    ///
    /// let regcomp_flags = RegcompFlags::new()
    ///     .add(RegcompFlags::EXTENDED)
    ///     .add(RegcompFlags::ICASE);
    /// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    ///
    /// let compiled_reg = Regex::new("^(hello).*(world)$", regcomp_flags)?;
    /// let matches = compiled_reg.regexec_bytes(b"hello world", 2, regexec_flags)?;
    ///
    /// for (i, matched) in matches.into_iter().enumerate() {
    ///     match matched {
    ///         Some(substr) => println!(
    ///             "Match {i}: {}",
    ///             std::str::from_utf8(substr.as_ref()).unwrap()
    ///         ),
    ///         None => println!("Match {i}: <None>"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn regexec_bytes<'a>(
        &self,
        data: &'a [u8],
        nmatches: usize,
        flags: RegexecFlags,
    ) -> Result<RegMatchBytes<'a>> {
        let Some(compiled_reg_obj) = self.get() else {
            return Err(RegexError::new(
                ErrorKind::Binding(BindingErrorCode::REGEX_VACANT),
                "Attempted to unwrap a vacant Regex object",
            ));
        };
        let mut match_vec: Vec<tre::regmatch_t> =
            vec![tre::regmatch_t { rm_so: 0, rm_eo: 0 }; nmatches];

        // SAFETY: compiled_reg is a wrapped type (see safety concerns for Regex). data is read-only.
        // match_vec has enough room for everything. flags also cannot wrap around.
        let result = unsafe {
            tre::tre_regnexecb(
                compiled_reg_obj,
                data.as_ptr().cast::<std::ffi::c_char>(),
                data.len(),
                nmatches,
                match_vec.as_mut_ptr(),
                flags.get(),
            )
        };
        if result != 0 {
            return Err(self.regerror(result));
        }

        let mut result: Vec<Option<Cow<'a, [u8]>>> = Vec::with_capacity(nmatches);
        for pmatch in match_vec {
            if pmatch.rm_so < 0 || pmatch.rm_eo < 0 {
                result.push(None);
                continue;
            }

            let start_offset = match_offset(pmatch.rm_so)?;
            let end_offset = match_offset(pmatch.rm_eo)?;

            result.push(Some(Cow::Borrowed(&data[start_offset..end_offset])));
        }

        Ok(result)
    }
}

/// Performs a regex search on the passed string, returning `nmatches` results.
///
/// This is a thin wrapper around [`Regex::regexec`].
///
/// Non-matching subexpressions or patterns will return `None` in the results.
///
/// # Arguments
/// * `compiled_reg`: the compiled [`Regex`] object.
/// * `string`: string to match against `compiled_reg`
/// * `nmatches`: number of matches to return
/// * `flags`: [`RegexecFlags`] to pass to [`tre_regnexec`](tre_regex_sys::tre_regnexec).
///
/// # Returns
/// If no error was found, a [`Vec`] of [`Option`]s will be returned.
///
/// If a given match index is empty, The `Option` will be `None`. Otherwise, [`Result`]s will be
/// returned, containing either errors or substrings of the matches. Errors may be returned due to
/// decoding problems, such as split codepoints.
///
/// # Errors
/// If an error is encountered during matching, it returns a [`RegexError`]. Match results may also
/// return errors, if decoding into UTF-8 was unsuccessful for whatever reason.
///
/// # Caveats
/// Unless copied, the match results must live at least as long as `string`. This is because they are
/// slices into `string` under the hood, for efficiency.
///
/// # Examples
/// ```
/// # use tre_regex::Result;
/// # fn main() -> Result<()> {
/// use tre_regex::{RegcompFlags, RegexecFlags, regcomp, regexec};
///
/// let regcomp_flags = RegcompFlags::new()
///     .add(RegcompFlags::EXTENDED)
///     .add(RegcompFlags::ICASE)
///     .add(RegcompFlags::UNGREEDY);
/// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
///
/// let compiled_reg = regcomp("^(hello).*(world)$", regcomp_flags)?;
/// let matches = regexec(
///     &compiled_reg,  // Compiled regex
///     "hello world",  // String to match against
///     2,              // Number of matches
///     regexec_flags   // Flags
/// )?;
///
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
#[inline]
pub fn regexec<'a>(
    compiled_reg: &Regex,
    string: &'a str,
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegMatchStr<'a>> {
    compiled_reg.regexec(string, nmatches, flags)
}

/// Performs a regex search on the passed bytes, returning `nmatches` results.
///
/// This is a thin wrapper around [`Regex::regexec_bytes`].
///
/// This function should only be used if you need to match raw bytes, or bytes which may not be
/// UTF-8 compliant. Otherwise, [`regexec`] is recommended instead.
///
/// # Arguments
/// * `compiled_reg`: the compiled [`Regex`] object.
/// * `data`: [`u8`] slice to match against `compiled_reg`
/// * `nmatches`: number of matches to return
/// * `flags`: [`RegexecFlags`] to pass to [`tre_regnexecb`](tre_regex_sys::tre_regnexecb).
///
/// # Returns
/// If no error was found, a [`Vec`] of [`Option`]s will be returned.
///
/// If a given match index is empty, The `Option` will be `None`. Otherwise, [`u8`] slices will be
/// returned.
///
/// # Errors
/// If an error is encountered during matching, it returns a [`RegexError`].
///
/// # Caveats
/// Unless copied, the match results must live at least as long as `data`. This is because they are
/// slices into `data` under the hood, for efficiency.
///
/// # Examples
/// ```
/// # use tre_regex::Result;
/// # fn main() -> Result<()> {
/// use tre_regex::{RegcompFlags, RegexecFlags, regcomp_bytes, regexec_bytes};
///
/// let regcomp_flags = RegcompFlags::new()
///     .add(RegcompFlags::EXTENDED)
///     .add(RegcompFlags::ICASE)
///     .add(RegcompFlags::UNGREEDY);
/// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
///
/// let compiled_reg = regcomp_bytes(b"^(hello).*(world)$", regcomp_flags)?;
/// let matches = regexec_bytes(
///     &compiled_reg,  // Compiled regex
///     b"hello world", // Bytes to match against
///     2,              // Number of matches
///     regexec_flags   // Flags
/// )?;
///
/// for (i, matched) in matches.into_iter().enumerate() {
///     match matched {
///         Some(substr) => println!(
///             "Match {i}: {}",
///             std::str::from_utf8(substr.as_ref()).unwrap()
///         ),
///         None => println!("Match {i}: <None>"),
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn regexec_bytes<'a>(
    compiled_reg: &Regex,
    data: &'a [u8],
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegMatchBytes<'a>> {
    compiled_reg.regexec_bytes(data, nmatches, flags)
}
