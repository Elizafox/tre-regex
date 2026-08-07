// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

use std::ffi::c_int;

use crate::{
    Regex, RegexecFlags,
    err::{RegexError, Result},
    exec::match_offset,
    tre,
};

pub type RegApproxMatchStr<'a> = RegApproxMatch<'a, str>;
pub type RegApproxMatchBytes<'a> = RegApproxMatch<'a, [u8]>;

/// Regex params passed to approximate matching functions such as [`regaexec`]
#[cfg(feature = "approx")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct RegApproxParams {
    cost_ins: i32,
    cost_del: i32,
    cost_subst: i32,
    max_cost: i32,
    max_ins: i32,
    max_del: i32,
    max_subst: i32,
    max_err: i32,
}

impl RegApproxParams {
    /// Creates a new empty [`RegApproxParams`] object.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            cost_ins: 0,
            cost_del: 0,
            cost_subst: 0,
            max_cost: 0,
            max_ins: 0,
            max_del: 0,
            max_subst: 0,
            max_err: 0,
        }
    }

    /// Sets the [`cost_ins`](tre_regex_sys::regaparams_t::cost_ins) element.
    #[must_use]
    #[inline]
    pub const fn cost_ins(self, cost_ins: i32) -> Self {
        let mut copy = self;
        copy.cost_ins = cost_ins;
        copy
    }

    /// Sets the [`cost_del`](tre_regex_sys::regaparams_t::cost_del) element.
    #[must_use]
    #[inline]
    pub const fn cost_del(self, cost_del: i32) -> Self {
        let mut copy = self;
        copy.cost_del = cost_del;
        copy
    }

    /// Sets the [`cost_subst`](tre_regex_sys::regaparams_t::cost_subst) element.
    #[must_use]
    #[inline]
    pub const fn cost_subst(self, cost_subst: i32) -> Self {
        let mut copy = self;
        copy.cost_subst = cost_subst;
        copy
    }

    /// Sets the [`max_cost`](tre_regex_sys::regaparams_t::max_cost) element.
    #[must_use]
    #[inline]
    pub const fn max_cost(self, max_cost: i32) -> Self {
        let mut copy = self;
        copy.max_cost = max_cost;
        copy
    }

    /// Sets the [`max_ins`](tre_regex_sys::regaparams_t::max_ins) element.
    #[must_use]
    #[inline]
    pub const fn max_ins(self, max_ins: i32) -> Self {
        let mut copy = self;
        copy.max_ins = max_ins;
        copy
    }

    /// Sets the [`max_del`](tre_regex_sys::regaparams_t::max_del) element.
    #[must_use]
    #[inline]
    pub const fn max_del(self, max_del: i32) -> Self {
        let mut copy = self;
        copy.max_del = max_del;
        copy
    }

    /// Sets the [`max_subst`](tre_regex_sys::regaparams_t::max_subst) element.
    #[must_use]
    #[inline]
    pub const fn max_subst(self, max_subst: i32) -> Self {
        let mut copy = self;
        copy.max_subst = max_subst;
        copy
    }

    /// Sets the [`max_err`](tre_regex_sys::regaparams_t::max_err) element.
    #[must_use]
    #[inline]
    pub const fn max_err(self, max_err: i32) -> Self {
        let mut copy = self;
        copy.max_err = max_err;
        copy
    }

    pub(crate) fn to_raw(self) -> Result<tre::regaparams_t> {
        fn convert(value: i32) -> Result<c_int> {
            c_int::try_from(value).map_err(|_| RegexError::InvalidApproxParameter)
        }

        Ok(tre::regaparams_t {
            cost_ins: convert(self.cost_ins)?,
            cost_del: convert(self.cost_del)?,
            cost_subst: convert(self.cost_subst)?,
            max_cost: convert(self.max_cost)?,
            max_ins: convert(self.max_ins)?,
            max_del: convert(self.max_del)?,
            max_subst: convert(self.max_subst)?,
            max_err: convert(self.max_err)?,
        })
    }
}

/// This struct is returned by [`regaexec`].
///
/// The match results from this function are very complex. See the [TRE documentation] for details
/// on how this all works and corresponding fields, and what they mean.
///
/// This structure should never be instantiated outside the library.
///
/// [TRE documentation]: <https://laurikari.net/tre/documentation/regaexec/>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegApproxMatch<'a, T: ?Sized> {
    data: &'a T,
    matches: Vec<Option<&'a T>>,
    cost: i32,
    num_ins: i32,
    num_del: i32,
    num_subst: i32,
}

impl<'a, T: ?Sized> RegApproxMatch<'a, T> {
    pub(crate) const fn new(
        data: &'a T,
        matches: Vec<Option<&'a T>>,
        amatch: tre::regamatch_t,
    ) -> Self {
        Self {
            data,
            matches,
            cost: amatch.cost,
            num_ins: amatch.num_ins,
            num_del: amatch.num_del,
            num_subst: amatch.num_subst,
        }
    }

    /// Gets the cost of the match
    #[must_use]
    pub const fn cost(&self) -> i32 {
        self.cost
    }

    /// Gets the number of insertions if the match
    #[must_use]
    pub const fn num_ins(&self) -> i32 {
        self.num_ins
    }

    /// Gets the number of deletions if the match
    #[must_use]
    pub const fn num_del(&self) -> i32 {
        self.num_del
    }

    /// Get the number of substitutions in the match
    #[must_use]
    pub const fn num_subst(&self) -> i32 {
        self.num_subst
    }

    /// Gets an immutable reference to the underlying data
    #[must_use]
    pub const fn get_orig_data(&self) -> &'a T {
        self.data
    }

    /// Gets the matches returned by this, as references to the data
    #[must_use]
    pub fn get_matches(&self) -> &[Option<&'a T>] {
        &self.matches
    }

    /// Consumes this result and returns its matches.
    #[must_use]
    pub fn into_matches(self) -> Vec<Option<&'a T>> {
        self.matches
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
    ///         Some(substr) => println!("Match {i}: {substr}"),
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
        let Some(compiled_reg_obj) = self.as_raw() else {
            return Err(RegexError::VacantRegex);
        };
        let data = string.as_bytes();
        let mut match_vec = vec![tre::regmatch_t::default(); nmatches];
        let mut amatch = tre::regamatch_t {
            nmatch: nmatches,
            pmatch: match_vec.as_mut_ptr(),
            ..Default::default()
        };

        // SAFETY: the regex is initialised, data is valid for its supplied length, and amatch
        // points to nmatches writable entries.
        let result_code = unsafe {
            tre::tre_reganexec(
                compiled_reg_obj,
                data.as_ptr().cast(),
                data.len(),
                &raw mut amatch,
                params.to_raw()?,
                flags.bits(),
            )
        };
        if result_code != 0 {
            return Err(self.regerror(result_code));
        }

        let mut result = Vec::with_capacity(nmatches);
        for pmatch in match_vec {
            if pmatch.rm_so < 0 || pmatch.rm_eo < 0 {
                result.push(None);
                continue;
            }

            let start_offset = match_offset(pmatch.rm_so)?;
            let end_offset = match_offset(pmatch.rm_eo)?;
            let matched = string
                .get(start_offset..end_offset)
                .ok_or(RegexError::InvalidUtf8Boundary)?;
            result.push(Some(matched));
        }

        Ok(RegApproxMatchStr::new(string, result, amatch))
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
    /// * `flags`: [`RegexecFlags`] to pass to [`tre_regaexecb`](tre_regex_sys::tre_regaexecb).
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
    /// slices into `data` under the hood, for efficiency. TRE's approximate byte matcher is
    /// NUL-terminated, so an embedded NUL marks the end of the searchable data.
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
    /// let compiled_reg = Regex::new_bytes(b"^(hello).*(world)$", regcomp_flags)?;
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
        let Some(compiled_reg_obj) = self.as_raw() else {
            return Err(RegexError::VacantRegex);
        };
        let mut match_vec: Vec<tre::regmatch_t> =
            vec![tre::regmatch_t { rm_so: 0, rm_eo: 0 }; nmatches];
        let mut amatch = tre::regamatch_t {
            nmatch: nmatches,
            pmatch: match_vec.as_mut_ptr(),
            ..Default::default()
        };
        let mut nul_terminated_data = Vec::with_capacity(data.len() + 1);
        nul_terminated_data.extend_from_slice(data);
        nul_terminated_data.push(0);

        // SAFETY: compiled_reg is initialised, nul_terminated_data has an explicit trailing NUL,
        // and amatch points to nmatches writable entries.
        let result = unsafe {
            tre::tre_regaexecb(
                compiled_reg_obj,
                nul_terminated_data.as_ptr().cast(),
                &raw mut amatch,
                params.to_raw()?,
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
/// * `flags`: [`RegexecFlags`] to pass to [`tre_reganexec`](tre_regex_sys::tre_reganexec).
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
///         Some(substr) => println!("Match {i}: {substr}"),
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
/// * `flags`: [`RegexecFlags`] to pass to [`tre_regaexecb`](tre_regex_sys::tre_regaexecb).
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
/// slices into `data` under the hood, for efficiency. TRE's approximate byte matcher is
/// NUL-terminated, so an embedded NUL marks the end of the searchable data.
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
/// let compiled_reg = Regex::new_bytes(b"^(hello).*(world)$", regcomp_flags)?;
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
