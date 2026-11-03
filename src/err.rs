// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

use std::ffi::{CString, c_char, c_int, c_uint};
use std::ptr::null_mut;

use crate::{Regex, tre};

// Public types
pub type ErrorInt = c_int;
pub type Result<T> = std::result::Result<T, RegexError>;

/// Error type returned in results
#[derive(Debug)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
#[non_exhaustive]
pub enum RegexError {
    /// An error reported by TRE.
    #[error("{message} (TRE code {code:?})")]
    Tre {
        /// TRE error code.
        code: tre::reg_errcode_t,

        /// Human-readable message returned by TRE.
        message: String,
    },

    /// An operation required a populated [`Regex`].
    #[error("attempted to use a vacant Regex object")]
    VacantRegex,

    /// TRE returned a malformed error buffer.
    #[error("could not convert TRE's error buffer to a C string")]
    InvalidErrorBuffer {
        /// Conversion error.
        #[source]
        source: std::ffi::FromVecWithNulError,
    },

    /// TRE's error message was not valid UTF-8.
    #[error("TRE returned an error message that is not valid UTF-8")]
    InvalidErrorMessage {
        /// Conversion error.
        #[source]
        source: std::ffi::IntoStringError,
    },

    /// TRE returned match offsets that are not UTF-8 character boundaries.
    #[error("TRE returned match offsets that are not UTF-8 character boundaries")]
    InvalidUtf8Boundary,

    /// TRE returned a match offset that cannot be represented by [`usize`].
    #[error("TRE returned an invalid match offset")]
    InvalidMatchOffset {
        /// Integer conversion error.
        #[source]
        source: std::num::TryFromIntError,
    },

    /// An approximate matching parameter cannot be represented by TRE's C integer type.
    #[cfg(feature = "approx")]
    #[error("approximate matching parameter is out of range")]
    InvalidApproxParameter,
}

fn error_buffer_to_string(errbuf: Vec<u8>) -> Result<String> {
    let errstr = CString::from_vec_with_nul(errbuf)
        .map_err(|source| RegexError::InvalidErrorBuffer { source })?;
    errstr
        .into_string()
        .map_err(|source| RegexError::InvalidErrorMessage { source })
}

impl Regex {
    /// Given the [`ErrorInt`] code and this object, build a [`RegexError`].
    ///
    /// # Arguments
    /// * `result`: the TRE result code, see [`reg_errcode_t`](tre_regex_sys::reg_errcode_t).
    ///
    /// # Returns
    /// A [`RegexError`] object. If creating the object fails, there is no way to know for sure.
    /// Fortunately, this should be a nonexistent occurence if the API is used correctly.
    #[must_use]
    pub fn regerror(&self, result: ErrorInt) -> RegexError {
        // SAFETY: compiled_reg should be valid; see safety concerns for Regex.
        let Some(compiled_reg_obj) = self.as_raw() else {
            return RegexError::VacantRegex;
        };
        // SAFETY: compiled_reg_obj is initialised, and a null buffer with size zero asks TRE for
        // the required error-buffer size.
        let bufsize = unsafe { tre::tre_regerror(result, compiled_reg_obj, null_mut(), 0) };
        let mut errbuf = vec![0u8; bufsize];
        // SAFETY: compiled_reg should be valid; errbuf has enough room as validated above
        unsafe {
            tre::tre_regerror(
                result,
                compiled_reg_obj,
                errbuf.as_mut_ptr().cast::<c_char>(),
                bufsize,
            );
        }
        let errstr = match error_buffer_to_string(errbuf) {
            Ok(errstr) => errstr,
            Err(error) => return error,
        };

        let result = c_uint::from_ne_bytes(result.to_ne_bytes());
        RegexError::Tre {
            code: tre::reg_errcode_t(result),
            message: errstr,
        }
    }
}

/// Given a [`Regex`] struct and [`ErrorInt`] code, build a [`RegexError`].
///
/// This is a thin wrapper around [`Regex::regerror`].
///
/// # Arguments
/// * `compiled_reg`: the compiled `Regex` that triggered the error.
/// * `result`: the TRE result code, see [`reg_errcode_t`](tre_regex_sys::reg_errcode_t).
///
/// **WARNING**: you should rarely need to call this directly.
///
/// # Returns
/// A [`RegexError`] object. If creating the object fails, there is no way to know for sure.
/// Fortunately, this should be a nonexistent occurence if the API is used correctly.
///
/// # Examples
/// ```
/// use std::ffi::c_char;
/// use std::ptr::null_mut;
/// use tre_regex::{{tre::{tre_regcomp, regex_t}}, Regex, regerror};
///
/// let mut compiled_reg: regex_t = Default::default();
/// let result = unsafe {
///     tre_regcomp(&mut compiled_reg, b"[a\0".as_ptr().cast::<c_char>(), 0)
/// };
/// let regex_error = regerror(&unsafe { Regex::new_from(compiled_reg) }, result);
/// println!("Error with regex: {regex_error}");
/// ```
#[must_use]
pub fn regerror(compiled_reg: &Regex, result: ErrorInt) -> RegexError {
    compiled_reg.regerror(result)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{RegexError, error_buffer_to_string};

    #[test]
    fn malformed_error_buffers_are_rejected() {
        let missing_nul = error_buffer_to_string(b"error".to_vec()).expect_err("missing NUL");
        assert!(missing_nul.source().is_some());
        assert!(matches!(missing_nul, RegexError::InvalidErrorBuffer { .. }));

        let invalid_utf8 = error_buffer_to_string(vec![0xff, 0]).expect_err("invalid UTF-8");
        assert!(invalid_utf8.source().is_some());
        assert!(matches!(
            invalid_utf8,
            RegexError::InvalidErrorMessage { .. }
        ));
    }
}
