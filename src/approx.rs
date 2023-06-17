use std::borrow::Cow;
use std::ffi::c_int;
use std::hint::unreachable_unchecked;

use crate::{
    err::{BindingErrorCode, ErrorKind, RegexError, Result},
    tre, Regex, RegexecFlags,
};

pub type RegApproxMatchStr<'a> = RegApproxMatch<&'a str, Result<Cow<'a, str>>>;
pub type RegApproxMatchBytes<'a> = RegApproxMatch<&'a [u8], Cow<'a, [u8]>>;

/// Regex params passed to approximate matching functions such as [`regaexec`]
#[cfg(feature = "approx")]
#[derive(Copy, Clone, Debug)]
pub struct RegApproxParams(tre::regaparams_t);

impl RegApproxParams {
    /// Creates a new empty [`RegApproxParams`] object.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(tre::regaparams_t::default())
    }

    /// Sets the [`cost_ins`](tre_regex_sys::regaparams_t::cost_ins) element.
    #[must_use]
    #[inline]
    pub const fn cost_ins(&self, cost_ins: c_int) -> Self {
        let mut copy = *self;
        copy.0.cost_ins = cost_ins;
        copy
    }

    /// Sets the [`cost_del`](tre_regex_sys::regaparams_t::cost_del) element.
    #[must_use]
    #[inline]
    pub const fn cost_del(&self, cost_del: c_int) -> Self {
        let mut copy = *self;
        copy.0.cost_del = cost_del;
        copy
    }

    /// Sets the [`cost_subst`](tre_regex_sys::regaparams_t::cost_subst) element.
    #[must_use]
    #[inline]
    pub const fn cost_subst(&self, cost_subst: c_int) -> Self {
        let mut copy = *self;
        copy.0.cost_subst = cost_subst;
        copy
    }

    /// Sets the [`max_cost`](tre_regex_sys::regaparams_t::max_cost) element.
    #[must_use]
    #[inline]
    pub const fn max_cost(&self, max_cost: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_cost = max_cost;
        copy
    }

    /// Sets the [`max_ins`](tre_regex_sys::regaparams_t::max_ins) element.
    #[must_use]
    #[inline]
    pub const fn max_ins(&self, max_ins: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_ins = max_ins;
        copy
    }

    /// Sets the [`max_del`](tre_regex_sys::regaparams_t::max_del) element.
    #[must_use]
    #[inline]
    pub const fn max_del(&self, max_del: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_del = max_del;
        copy
    }

    /// Sets the [`max_subst`](tre_regex_sys::regaparams_t::max_subst) element.
    #[must_use]
    #[inline]
    pub const fn max_subst(&self, max_subst: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_subst = max_subst;
        copy
    }

    /// Sets the [`max_err`](tre_regex_sys::regaparams_t::max_err) element.
    #[must_use]
    #[inline]
    pub const fn max_err(&self, max_err: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_err = max_err;
        copy
    }

    /// Get an immutable reference to the underlying [`regaparams_t`](tre_regex_sys::regaparams_t) object.
    #[must_use]
    #[inline]
    pub const fn get(&self) -> &tre::regaparams_t {
        &self.0
    }

    /// Get a mutable reference to the underlying [`regaparams_t`](tre_regex_sys::regaparams_t) object.
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self) -> &mut tre::regaparams_t {
        &mut self.0
    }
}

impl Default for RegApproxParams {
    fn default() -> Self {
        Self::new()
    }
}

/// This struct is returned by [`regaexec`] and friends.
///
/// The match results from this function are very complex. See the [TRE documentation] for details
/// on how this all works and corresponding fields, and what they mean.
///
/// This structure should never be instantiated outside the library.
///
/// [TRE documentation]: <https://laurikari.net/tre/documentation/regaexec/>
#[derive(Clone, Debug)]
pub struct RegApproxMatch<Data, Res> {
    data: Data,
    matches: Vec<Option<Res>>,
    amatch: tre::regamatch_t,
}

impl<Data, Res> RegApproxMatch<Data, Res> {
    pub(crate) fn new(data: Data, matches: Vec<Option<Res>>, amatch: tre::regamatch_t) -> Self {
        Self {
            data,
            matches,
            amatch,
        }
    }

    /// Gets the cost of the match
    pub const fn cost(&self) -> c_int {
        self.amatch.cost
    }

    /// Gets the number of insertions if the match
    pub const fn num_ins(&self) -> c_int {
        self.amatch.num_ins
    }

    /// Gets the number of deletions if the match
    pub const fn num_del(&self) -> c_int {
        self.amatch.num_del
    }

    /// Get the number of substitutions in the match
    pub const fn num_subst(&self) -> c_int {
        self.amatch.num_subst
    }

    /// Gets an immutable reference to the underlying data
    pub const fn get_orig_data(&self) -> &Data {
        &self.data
    }

    /// Gets the matches returned by this, as references to the data
    pub const fn get_matches(&self) -> &Vec<Option<Res>> {
        &self.matches
    }

    /// Gets a reference to the underlying [`regamatch_t`](tre_regex_sys::regamatch_t) object.
    pub const fn get_regamatch(&self) -> &tre::regamatch_t {
        &self.amatch
    }
}

impl Regex {
    /// Performs an approximate regex search on the passed string, returning `nmatches` results.
    ///
    /// Non-matching subexpressions or patterns will return `None` in the results.
    ///
    /// # Arguments
    /// * `string`: string to match against `compiled_reg`
    /// * `params`: see [`RegApproxParams`]
    /// * `nmatches`: number of matches to return
    /// * `flags`: [`RegexecFlags`] to pass to [`tre_reganexec`](tre_regex_sys::tre_reganexec).
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
    /// use tre_regex::{RegcompFlags, RegexecFlags, RegApproxParams, Regex};
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
    /// let compiled_reg = Regex::new("^(hello).*(world)$", regcomp_flags)?;
    /// let result = compiled_reg.regaexec(
    ///     "hello world",      // String to match against
    ///     &regaexec_params,   // Matching parameters
    ///     3,                  // Number of matches we want
    ///     regaexec_flags      // Flags
    /// )?;
    ///
    /// for (i, matched) in result.get_matches().into_iter().enumerate() {
    ///     match matched {
    ///         Some(substr) => println!("Match {i}: {}", substr.as_ref().unwrap()),
    ///         None => println!("Match {i}: <None>"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn regaexec<'a>(
        &self,
        string: &'a str,
        params: &RegApproxParams,
        nmatches: usize,
        flags: RegexecFlags,
    ) -> Result<RegApproxMatchStr<'a>> {
        let data = string.as_bytes();
        let match_results = self.regaexec_bytes(data, params, nmatches, flags)?;

        let mut result: Vec<Option<Result<Cow<'a, str>>>> = Vec::with_capacity(nmatches);
        for pmatch in match_results.get_matches() {
            let Some(pmatch) = pmatch else { result.push(None); continue; };

            result.push(Some(match pmatch {
                Cow::Borrowed(pmatch) => match std::str::from_utf8(pmatch) {
                    Ok(s) => Ok(s.into()),
                    Err(e) => Err(RegexError::new(
                        ErrorKind::Binding(BindingErrorCode::ENCODING),
                        &format!("UTF-8 encoding error: {e}"),
                    )),
                },
                // SAFETY: cannot get here, we only have borrowed values.
                _ => unsafe { unreachable_unchecked() },
            }));
        }

        Ok(RegApproxMatchStr::new(
            string,
            result,
            *match_results.get_regamatch(),
        ))
    }

    /// Performs an approximate regex search on the passed bytes, returning `nmatches` results.
    ///
    /// This function should only be used if you need to match raw bytes, or bytes which may not be
    /// UTF-8 compliant. Otherwise, [`regaexec`] is recommended instead.
    ///
    /// # Arguments
    /// * `data`: [`u8`] slice to match against `compiled_reg`
    /// * `params`: see [`RegApproxParams`]
    /// * `nmatches`: number of matches to return
    /// * `flags`: [`RegexecFlags`] to pass to [`tre_reganexec`](tre_regex_sys::tre_reganexec).
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
    /// use tre_regex::{RegcompFlags, RegexecFlags, RegApproxParams, Regex};
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
    /// let compiled_reg = Regex::new("^(hello).*(world)$", regcomp_flags)?;
    /// let result = compiled_reg.regaexec_bytes(
    ///     b"hello world",     // Bytes to match against
    ///     &regaexec_params,   // Matching parameters
    ///     3,                  // Number of matches we want
    ///     regaexec_flags      // Flags
    /// )?;
    ///
    /// for (i, matched) in result.get_matches().into_iter().enumerate() {
    ///     match matched {
    ///         Some(substr) => println!(
    ///             "Match {i}: {}",
    ///             std::str::from_utf8(substr).unwrap()
    ///         ),
    ///         None => println!("Match {i}: <None>"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn regaexec_bytes<'a>(
        &self,
        data: &'a [u8],
        params: &RegApproxParams,
        nmatches: usize,
        flags: RegexecFlags,
    ) -> Result<RegApproxMatchBytes<'a>> {
        let Some(compiled_reg_obj) = self.get() else {
            return Err(RegexError::new(
                ErrorKind::Binding(BindingErrorCode::REGEX_VACANT),
                "Attempted to unwrap a vacant Regex object"
            ));
        };
        let mut match_vec: Vec<tre::regmatch_t> =
            vec![tre::regmatch_t { rm_so: 0, rm_eo: 0 }; nmatches];
        let mut amatch = tre::regamatch_t {
            nmatch: nmatches,
            pmatch: match_vec.as_mut_ptr(),
            ..Default::default()
        };

        // SAFETY: compiled_reg is a wrapped type (see safety concerns for Regex). data is read-only.
        // match_vec has enough room for everything. flags also cannot wrap around.
        #[allow(clippy::cast_possible_wrap)]
        let result = unsafe {
            tre::tre_reganexec(
                compiled_reg_obj,
                data.as_ptr().cast::<i8>(),
                data.len(),
                &mut amatch,
                *params.get(),
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

            // Wraparound is impossible.
            #[allow(clippy::cast_sign_loss)]
            let start_offset = pmatch.rm_so as usize;
            #[allow(clippy::cast_sign_loss)]
            let end_offset = pmatch.rm_eo as usize;

            result.push(Some(Cow::Borrowed(&data[start_offset..end_offset])));
        }

        Ok(RegApproxMatchBytes::new(data, result, amatch))
    }
}

/// Performs an approximate regex search on the passed string, returning `nmatches` results.
///
/// This is a thin wrapper around [`Regex::regaexec`].
///
/// Non-matching subexpressions or patterns will return `None` in the results.
///
/// # Arguments
/// * `compiled_reg`: the compiled [`Regex`] object.
/// * `string`: string to match against `compiled_reg`
/// * `params`: see [`RegApproxParams`]
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
/// use tre_regex::{RegcompFlags, RegexecFlags, RegApproxParams, Regex, regaexec};
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
/// let compiled_reg = Regex::new("^(hello).*(world)$", regcomp_flags)?;
/// let result = regaexec(
///     &compiled_reg,      // Compiled regex
///     "hello world",      // String to match against
///     &regaexec_params,   // Matching parameters
///     3,                  // Number of matches we want
///     regaexec_flags      // Flags
/// )?;
///
/// for (i, matched) in result.get_matches().into_iter().enumerate() {
///     match matched {
///         Some(substr) => println!("Match {i}: {}", substr.as_ref().unwrap()),
///         None => println!("Match {i}: <None>"),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn regaexec<'a>(
    compiled_reg: &Regex,
    string: &'a str,
    params: &RegApproxParams,
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegApproxMatchStr<'a>> {
    compiled_reg.regaexec(string, params, nmatches, flags)
}

/// Performs an approximate regex search on the passed bytes, returning `nmatches` results.
///
/// This is a thin wrapper around [`Regex::regaexec_bytes`].
///
/// This function should only be used if you need to match raw bytes, or bytes which may not be
/// UTF-8 compliant. Otherwise, [`regaexec`] is recommended instead.
///
/// # Arguments
/// * `compiled_reg`: the compiled [`Regex`] object.
/// * `data`: [`u8`] slice to match against `compiled_reg`
/// * `params`: see [`RegApproxParams`]
/// * `nmatches`: number of matches to return
/// * `flags`: [`RegexecFlags`] to pass to [`tre_regnexec`](tre_regex_sys::tre_regnexec).
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
/// use tre_regex::{RegcompFlags, RegexecFlags, RegApproxParams, Regex, regaexec_bytes};
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
/// let compiled_reg = Regex::new("^(hello).*(world)$", regcomp_flags)?;
/// let result = regaexec_bytes(
///     &compiled_reg,      // Compiled regex
///     b"hello world",     // Bytes to match against
///     &regaexec_params,   // Matching parameters
///     3,                  // Number of matches we want
///     regaexec_flags      // Flags
/// )?;
///
/// for (i, matched) in result.get_matches().into_iter().enumerate() {
///     match matched {
///         Some(substr) => println!(
///             "Match {i}: {}",
///             std::str::from_utf8(substr).unwrap()
///         ),
///         None => println!("Match {i}: <None>"),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn regaexec_bytes<'a>(
    compiled_reg: &Regex,
    data: &'a [u8],
    params: &RegApproxParams,
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegApproxMatchBytes<'a>> {
    compiled_reg.regaexec_bytes(data, params, nmatches, flags)
}
