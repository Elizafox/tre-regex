// SPDX-License-Identifier: BSD-2-Clause
// See LICENSE file in the project root for full license text.

use crate::{
    Regex,
    err::{BindingErrorCode, ErrorKind, RegexError, Result},
    flags::RegexecFlags,
    tre,
};

/// Captures returned from a UTF-8 match.
pub type RegMatchStr<'a> = Vec<Option<&'a str>>;
/// Captures returned from a byte match.
pub type RegMatchBytes<'a> = Vec<Option<&'a [u8]>>;

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
    /// Returns whether `string` matches this regular expression.
    ///
    /// # Errors
    /// Returns a [`RegexError`] for execution errors other than a normal non-match.
    pub fn is_match(&self, string: &str, flags: RegexecFlags) -> Result<bool> {
        match self.regexec(string, 0, flags) {
            Ok(_) => Ok(true),
            Err(RegexError {
                kind: ErrorKind::Tre(tre::reg_errcode_t::REG_NOMATCH),
                ..
            }) => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Returns whether `data` matches this byte regular expression.
    ///
    /// # Errors
    /// Returns a [`RegexError`] for execution errors other than a normal non-match.
    pub fn is_match_bytes(&self, data: &[u8], flags: RegexecFlags) -> Result<bool> {
        match self.regexec_bytes(data, 0, flags) {
            Ok(_) => Ok(true),
            Err(RegexError {
                kind: ErrorKind::Tre(tre::reg_errcode_t::REG_NOMATCH),
                ..
            }) => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Returns capture groups from `string`.
    ///
    /// This is the idiomatically named equivalent of [`regexec`](Self::regexec).
    ///
    /// # Errors
    /// Returns a [`RegexError`] if matching fails or a capture is not valid UTF-8.
    pub fn captures<'a>(
        &self,
        string: &'a str,
        capacity: usize,
        flags: RegexecFlags,
    ) -> Result<RegMatchStr<'a>> {
        self.regexec(string, capacity, flags)
    }

    /// Returns capture groups from `data`.
    ///
    /// This is the idiomatically named equivalent of [`regexec_bytes`](Self::regexec_bytes).
    ///
    /// # Errors
    /// Returns a [`RegexError`] if matching fails.
    pub fn captures_bytes<'a>(
        &self,
        data: &'a [u8],
        capacity: usize,
        flags: RegexecFlags,
    ) -> Result<RegMatchBytes<'a>> {
        self.regexec_bytes(data, capacity, flags)
    }

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
    /// If a given match index is empty, its `Option` is `None`; otherwise it contains a borrowed
    /// substring of the input.
    ///
    /// # Errors
    /// Returns a [`RegexError`] if matching fails or TRE returns offsets that do not fall on UTF-8
    /// character boundaries.
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
    ///         Some(substr) => println!("Match {i}: '{substr}'"),
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
        let Some(compiled_reg_obj) = self.as_raw() else {
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
                flags.bits(),
            )
        };
        if result != 0 {
            return Err(self.regerror(result));
        }

        let mut result = Vec::with_capacity(nmatches);
        for pmatch in match_vec {
            if pmatch.rm_so < 0 || pmatch.rm_eo < 0 {
                result.push(None);
                continue;
            }

            let start_offset = match_offset(pmatch.rm_so)?;
            let end_offset = match_offset(pmatch.rm_eo)?;
            let matched = string.get(start_offset..end_offset).ok_or_else(|| {
                RegexError::new(
                    ErrorKind::Binding(BindingErrorCode::ENCODING),
                    "TRE returned match offsets that are not UTF-8 character boundaries",
                )
            })?;
            result.push(Some(matched));
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
        let Some(compiled_reg_obj) = self.as_raw() else {
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
                flags.bits(),
            )
        };
        if result != 0 {
            return Err(self.regerror(result));
        }

        let mut result = Vec::with_capacity(nmatches);
        for pmatch in match_vec {
            if pmatch.rm_so < 0 || pmatch.rm_eo < 0 {
                result.push(None);
                continue;
            }

            let start_offset = match_offset(pmatch.rm_so)?;
            let end_offset = match_offset(pmatch.rm_eo)?;

            result.push(Some(&data[start_offset..end_offset]));
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
/// If a given match index is empty, its `Option` is `None`; otherwise it contains a borrowed
/// substring of the input.
///
/// # Errors
/// Returns a [`RegexError`] if matching fails or TRE returns offsets that do not fall on UTF-8
/// character boundaries.
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
///         Some(substr) => println!("Match {i}: '{substr}'"),
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
