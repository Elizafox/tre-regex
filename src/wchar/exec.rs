use std::borrow::Cow;

use widestring::WideStr;

use crate::{err::{BindingErrorCode, ErrorKind, RegexError, Result}, flags::RegexecFlags, tre, Regex};

pub type RegMatchWideStr<'a> = Vec<Option<Cow<'a, WideStr>>>;

impl Regex {
    /// Performs a regex search on the passed wide string, returning `nmatches` results.
    ///
    /// This function should only be used if you need to match wide strings. Otherwise, [`regexec`]
    /// is recommended instead.
    ///
    /// # Arguments
    /// * `string`: [`WideStr`] to match against `compiled_reg`
    /// * `nmatches`: number of matches to return
    /// * `flags`: [`RegexecFlags`] to pass to [`tre_regnexec`](tre_regex_sys::tre_regnexec).
    ///
    /// # Returns
    /// If no error was found, a [`Vec`] of [`Option`]s will be returned.
    ///
    /// If a given match index is empty, The `Option` will be `None`. Otherwise, the `Option` will
    /// contain a [`WideStr`].
    ///
    /// # Errors
    /// If an error is encountered during matching, it returns a [`RegexError`].
    ///
    /// # Caveats
    /// Unless copied, the match results must live at least as long as `string`. This is because
    /// they are slices into `string` under the hood, for efficiency.
    ///
    /// # Examples
    /// ```
    /// # use tre_regex::Result;
    /// # fn main() -> Result<()> {
    /// use tre_regex::{RegcompFlags, RegexecFlags, Regex};
    /// use widestring::widestr;
    ///
    /// let regcomp_flags = RegcompFlags::new()
    ///     .add(RegcompFlags::EXTENDED)
    ///     .add(RegcompFlags::ICASE);
    /// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    ///
    /// let compiled_reg = Regex::new_wide(widestr!("^(hello).*(world)$"), regcomp_flags)?;
    /// let matches = compiled_reg.regwexec(widestr!("hello world"), 2, regexec_flags)?;
    ///
    /// for (i, matched) in matches.into_iter().enumerate() {
    ///     match matched {
    ///         Some(substr) => println!("Match {i}: {}", substr.display()),
    ///         None => println!("Match {i}: <None>"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`regexec`]: crate::regexec
    pub fn regwexec<'a>(
        &self,
        string: &'a WideStr,
        nmatches: usize,
        flags: RegexecFlags,
    ) -> Result<RegMatchWideStr<'a>> {
        let Some(compiled_reg_obj) = self.get() else {
            return Err(RegexError::new(
                ErrorKind::Binding(BindingErrorCode::REGEX_VACANT),
                "Attempted to unwrap a vacant Regex object",
            ));
        };
        let mut match_vec: Vec<tre::regmatch_t> =
            vec![tre::regmatch_t { rm_so: 0, rm_eo: 0 }; nmatches];

        // SAFETY: compiled_reg is a wrapped type (see safety concerns for Regex). string is read-only.
        // match_vec has enough room for everything. flags also cannot wrap around.
        #[allow(clippy::cast_possible_wrap)]
        let result = unsafe {
            tre::tre_regwnexec(
                compiled_reg_obj,
                string.as_ptr().cast(),
                string.len(),
                nmatches,
                match_vec.as_mut_ptr(),
                flags.get(),
            )
        };
        if result != 0 {
            return Err(self.regerror(result));
        }

        let mut result: Vec<Option<Cow<'a, WideStr>>> = Vec::with_capacity(nmatches);
        for pmatch in match_vec {
            if pmatch.rm_so < 0 || pmatch.rm_eo < 0 {
                result.push(None);
                continue;
            }

            // Wraparound is impossible.
            #[allow(clippy::cast_sign_loss)]
            let start_offset = pmatch.rm_so as usize;
            #[allow(clippy::cast_sign_loss)]
            let end_offset = pmatch.rm_eo as usize;

            result.push(Some(Cow::Borrowed(&string[start_offset..end_offset])));
        }

        Ok(result)
    }
}

/// Performs a regex search on the passed wide string, returning `nmatches` results.
///
/// This is a thin wrapper around [`Regex::regwexec`].
///
/// This function should only be used if you need to match wide strings.
///
/// # Arguments
/// * `compiled_reg`: the compiled [`Regex`] object.
/// * `string`: [`WideStr`] to match against `compiled_reg`
/// * `nmatches`: number of matches to return
/// * `flags`: [`RegexecFlags`] to pass to [`tre_regwnexec`](tre_regex_sys::tre_regwnexec).
///
/// # Returns
/// If no error was found, a [`Vec`] of [`Option`]s will be returned.
///
/// If a given match index is empty, The `Option` will be `None`. Otherwise, a [`WideStr`]
/// will be returned.
///
/// # Errors
/// If an error is encountered during matching, it returns a [`RegexError`].
///
/// # Caveats
/// Unless copied, the match results must live at least as long as `string`. This is because they are
/// slices into `string` under the hood, for efficiency.
///
/// # Examples
/// ```
/// # use tre_regex::Result;
/// # fn main() -> Result<()> {
/// use tre_regex::{RegcompFlags, RegexecFlags, regwcomp, regwexec};
/// use widestring::widestr;
///
/// let regcomp_flags = RegcompFlags::new()
///     .add(RegcompFlags::EXTENDED)
///     .add(RegcompFlags::ICASE)
///     .add(RegcompFlags::UNGREEDY);
/// let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
///
/// let compiled_reg = regwcomp(widestr!("^(hello).*(world)$"), regcomp_flags)?;
/// let matches = regwexec(
///     &compiled_reg,              // Compiled regex
///     widestr!("hello world"),    // String to match against
///     2,                          // Number of matches
///     regexec_flags               // Flags
/// )?;
///
/// for (i, matched) in matches.into_iter().enumerate() {
///     match matched {
///         Some(substr) => println!("Match {i}: {}", substr.display()),
///         None => println!("Match {i}: <None>"),
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn regwexec<'a>(
    compiled_reg: &Regex,
    string: &'a WideStr,
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegMatchWideStr<'a>> {
    compiled_reg.regwexec(string, nmatches, flags)
}
