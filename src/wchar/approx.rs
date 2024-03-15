use std::borrow::Cow;

use widestring::WideStr;

use crate::{
    err::{BindingErrorCode, ErrorKind, RegexError, Result},
    tre, RegApproxMatch, RegApproxParams, Regex, RegexecFlags,
};

pub type RegApproxMatchWideStr<'a> = RegApproxMatch<&'a WideStr, Cow<'a, WideStr>>;

impl Regex {
    /// Performs an approximate regex search on the passed wide string, returning `nmatches`
    /// results.
    ///
    /// This function should only be used if you need to match raw wide string. Otherwise,
    /// [`regaexec`] is recommended instead.
    ///
    /// # Arguments
    /// * `string`: [`WideStr`] to match against `compiled_reg`
    /// * `params`: see [`RegApproxParams`]
    /// * `nmatches`: number of matches to return
    /// * `flags`: [`RegexecFlags`] to pass to [`tre_reganexec`](tre_regex_sys::tre_reganexec).
    ///
    /// # Returns
    /// If no error was found, a [`Vec`] of [`Option`]s will be returned.
    ///
    /// If a given match index is empty, The `Option` will be `None`. Otherwise, a [`WideStr`] will
    /// be returned.
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
    /// use tre_regex::{RegcompFlags, RegexecFlags, RegApproxParams, Regex};
    /// use widestring::widestr;
    ///
    /// let regcomp_flags = RegcompFlags::new()
    ///     .add(RegcompFlags::EXTENDED)
    ///     .add(RegcompFlags::ICASE);
    /// let regaexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    /// let regaexec_params = RegApproxParams::new()
    ///     .cost_ins(1)
    ///     .cost_del(1)
    ///     .cost_subst(1)
    ///     .max_cost(2)
    ///     .max_del(2)
    ///     .max_ins(2)
    ///     .max_subst(2)
    ///     .max_err(2);
    ///
    /// let compiled_reg = Regex::new_wide(widestr!("^(hello).*(world)$"), regcomp_flags)?;
    /// let result = compiled_reg.regawexec(
    ///     widestr!("hello world"),    // Bytes to match against
    ///     &regaexec_params,           // Matching parameters
    ///     3,                          // Number of matches we want
    ///     regaexec_flags              // Flags
    /// )?;
    ///
    /// for (i, matched) in result.get_matches().into_iter().enumerate() {
    ///     match matched {
    ///         Some(substr) => println!("Match {i}: {}", substr.display()),
    ///         None => println!("Match {i}: <None>"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`regaexec`]: crate::Regex::regaexec
    pub fn regawexec<'a>(
        &self,
        string: &'a WideStr,
        params: &RegApproxParams,
        nmatches: usize,
        flags: RegexecFlags,
    ) -> Result<RegApproxMatchWideStr<'a>> {
        let Some(compiled_reg_obj) = self.get() else {
            return Err(RegexError::new(
                ErrorKind::Binding(BindingErrorCode::REGEX_VACANT),
                "Attempted to unwrap a vacant Regex object",
            ));
        };
        let mut match_vec: Vec<tre::regmatch_t> =
            vec![tre::regmatch_t { rm_so: 0, rm_eo: 0 }; nmatches];
        let mut amatch = tre::regamatch_t {
            nmatch: nmatches,
            pmatch: match_vec.as_mut_ptr(),
            ..Default::default()
        };

        // SAFETY: compiled_reg is a wrapped type (see safety concerns for Regex). string is read-only.
        // match_vec has enough room for everything. flags also cannot wrap around.
        #[allow(clippy::cast_possible_wrap)]
        let result = unsafe {
            tre::tre_regawnexec(
                compiled_reg_obj,
                string.as_ptr().cast(),
                string.len(),
                &mut amatch,
                *params.get(),
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

        Ok(RegApproxMatchWideStr::new(string, result, amatch))
    }
}

/// Performs an approximate regex search on the passed wide string, returning `nmatches` results.
///
/// This is a thin wrapper around [`Regex::regawexec`].
///
/// Non-matching subexpressions or patterns will return `None` in the results.
///
/// # Arguments
/// * `compiled_reg`: the compiled [`Regex`] object.
/// * `string`: [`WideStr`] to match against `compiled_reg`
/// * `params`: see [`RegApproxParams`]
/// * `nmatches`: number of matches to return
/// * `flags`: [`RegexecFlags`] to pass to [`tre_regnexec`](tre_regex_sys::tre_regnexec).
///
/// # Returns
/// If no error was found, a [`Vec`] of [`Option`]s will be returned.
///
/// If a given match index is empty, The `Option` will be `None`. Otherwise, a [`WideStr`] will be
/// returned.
///
/// # Errors
/// If an error is encountered during matching, it returns a [`RegexError`].
///
/// # Caveats
/// Unless copied, the match results must live at least as long as `string`. This is because they
/// are slices into `string` under the hood, for efficiency.
///
/// # Examples
/// ```
/// # use tre_regex::Result;
/// # fn main() -> Result<()> {
/// use tre_regex::{RegcompFlags, RegexecFlags, RegApproxParams, Regex, regawexec};
/// use widestring::widestr;
///
/// let regcomp_flags = RegcompFlags::new()
///     .add(RegcompFlags::EXTENDED)
///     .add(RegcompFlags::ICASE);
/// let regaexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
/// let regaexec_params = RegApproxParams::new()
///     .cost_ins(1)
///     .cost_del(1)
///     .cost_subst(1)
///     .max_cost(2)
///     .max_del(2)
///     .max_ins(2)
///     .max_subst(2)
///     .max_err(2);
///
/// let compiled_reg = Regex::new_wide(widestr!("^(hello).*(world)$"), regcomp_flags)?;
/// let result = regawexec(
///     &compiled_reg,              // Compiled regex
///     widestr!("hello world"),    // String to match against
///     &regaexec_params,           // Matching parameters
///     3,                          // Number of matches we want
///     regaexec_flags              // Flags
/// )?;
///
/// for (i, matched) in result.get_matches().into_iter().enumerate() {
///     match matched {
///         Some(substr) => println!("Match {i}: {}", substr.display()),
///         None => println!("Match {i}: <None>"),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn regawexec<'a>(
    compiled_reg: &Regex,
    string: &'a WideStr,
    params: &RegApproxParams,
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegApproxMatchWideStr<'a>> {
    compiled_reg.regawexec(string, params, nmatches, flags)
}
