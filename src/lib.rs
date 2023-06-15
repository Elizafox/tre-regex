//! These are safe bindings to the [`tre_regex_sys`] module.
//!
//! These bindings are designed to provide an idiomatic Rust-like API to the [TRE library] as much
//! as possible. Most of the TRE API is suported, except the `wchar_t` functionality (as `wchar_t`
//! is technically standard, but is 16-bit on Windows and 32-bit almost everywhere else).
//!
//! # Examples
//! Two API's are presented: the function API, and the object API. Whichever one you choose to use
//! is up to you, although the function API is implemented as a thin wrapper around the object API.
//!
//! ## Object API
//! ```
//! # use tre_regex::Result;
//! # fn main() -> Result<()> {
//! use tre_regex::{RegcompFlags, RegexecFlags, Regex};
//!
//! let regcomp_flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
//! let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
//!
//! let compiled_reg = Regex::new("(([[:alpha:]]+).*)*", regcomp_flags)?;
//! let matches = compiled_reg.regexec("hello world", 2, regexec_flags)?;
//!
//! for (i, matched) in matches.into_iter().enumerate() {
//!     match matched {
//!         Some(res) => {
//!             match res {
//!                 Ok(substr) => println!("Match {i}: '{}'", substr),
//!                 Err(e) => println!("Match {i}: <Error: {e}>"),
//!             }
//!         },
//!         None => println!("Match {i}: <None>"),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Function API
//! ```
//! # use tre_regex::Result;
//! # fn main() -> Result<()> {
//! use tre_regex::{RegcompFlags, RegexecFlags, regcomp, regexec};
//!
//! let regcomp_flags = RegcompFlags::new().add(RegcompFlags::EXTENDED);
//! let regexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
//!
//! let compiled_reg = regcomp("(([[:alpha:]]+).*)*", regcomp_flags)?;
//! let matches = regexec(&compiled_reg, "hello world", 2, regexec_flags)?;
//!
//! for (i, matched) in matches.into_iter().enumerate() {
//!     match matched {
//!         Some(res) => {
//!             match res {
//!                 Ok(substr) => println!("Match {i}: '{}'", substr),
//!                 Err(e) => println!("Match {i}: <Error: {e}>"),
//!             }
//!         },
//!         None => println!("Match {i}: <None>"),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! [TRE library]: <https://laurikari.net/tre/>

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

/// Public re-export of the [`tre_regex_sys`] module.
pub use tre_regex_sys as tre;

#[cfg(feature = "approx")]
mod approx;
mod comp;
mod err;
mod exec;
mod flags;
#[cfg(test)]
mod tests;

#[cfg(feature = "approx")]
pub use crate::approx::*;
pub use crate::comp::*;
pub use crate::err::*;
pub use crate::exec::*;
pub use crate::flags::*;

/// The base regex object.
///
/// This object takes care of freeing itself upon dropping, so you don't have to call
/// [`tre_regfree`](tre_regex_sys::tre_regfree) yourself.
///
/// This object provides an API similar to the function API. See the documentation on the
/// individual functions for more information.
#[derive(Debug)]
pub struct Regex(Option<tre::regex_t>);

impl Regex {
    /// Create a new [`Regex`] object from the given [`regex_t`](tre_regex_sys::regex_t).
    ///
    /// This function is for advanced use only. Don't mess with it unless you know exactly what you
    /// are doing.
    ///
    /// **WARNING**: Do **NOT** place a [`regex_t`](tre_regex_sys::regex_t) here that you didn't
    /// get from [`regcomp`] or [`tre_regcomp`](tre_regex_sys::tre_regcomp). Otherwise, when the
    /// [`Regex`] object drops, it will call [`tre_regfree`](tre_regex_sys::tre_regfree`) on memory
    /// not allocated by TRE itself. This is **undefined behaviour** and will likely cause a
    /// segfault. This is why the function is marked `unsafe`.
    ///
    /// # Arguments
    /// * `regex`: A [`regex_t`](tre_regex_sys::regex_t) to wrap.
    ///
    /// # Returns
    /// A new [`Regex`] object, containing the passed-in [`regex_t`](tre_regex_sys::regex_t).
    ///
    /// # Safety
    /// The `regex` parameter must have been initalised by [`tre_regcomp`](tre_regex_sys::tre_regcomp)
    /// or taken from another [`Regex`] object.
    ///
    /// [`regcomp`]: crate::regcomp
    #[must_use]
    #[inline]
    pub const unsafe fn new_from(regex: tre::regex_t) -> Self {
        Self(Some(regex))
    }

    /// Relinquish the underlying [`regex_t`](tre_regex_sys::regex_t) object.
    ///
    /// This is an advanced function and should not be used unless you know what you are doing.
    ///
    /// # Returns
    /// `None` if the object is vacant, otherwise `Some(`[`regex_t`](tre_regex_sys::regex_t)`)`.
    ///
    /// # Safety
    /// A leak could result if the object is not properly freed with
    /// [`tre_regfree`](tre_regex_sys::tre_regfree) if the object was initalised by the TRE API.
    #[must_use]
    #[inline]
    pub unsafe fn release(&mut self) -> Option<tre::regex_t> {
        let regex = self.0;
        self.0 = None;
        regex
    }

    /// Gets an immutable reference to the underlying [`regex_t`](tre_regex_sys::regex_t) object.
    #[must_use]
    #[inline]
    pub const fn get(&self) -> &Option<tre::regex_t> {
        &self.0
    }

    /// Gets a mutable reference to the underlying [`regex_t`](tre_regex_sys::regex_t) object.
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self) -> &mut Option<tre::regex_t> {
        &mut self.0
    }
}

impl Drop for Regex {
    /// Executes the destructor for this type.
    ///
    /// The destructor will call [`tre_regfree`](tre_regex_sys::tre_regfree) on the internal
    /// [`regex_t`](tre_regex_sys::regex_t).
    #[inline]
    fn drop(&mut self) {
        let Some(compiled_reg) = self.get_mut() else { return; };

        // SAFETY: freeing data passed into the struct previously.
        // If the data came from our API, this is safe. Otherwise, the user must opt into storing
        // the regex here.
        unsafe {
            tre::tre_regfree(compiled_reg);
        }
    }
}
