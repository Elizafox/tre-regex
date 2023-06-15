use std::ffi::c_int;

use crate::tre;

#[allow(clippy::module_name_repetitions)]
pub type RegFlags = c_int;

/// Flags to pass to [`regcomp`](crate::regcomp).
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug)]
pub struct RegcompFlags(RegFlags);

impl RegcompFlags {
    /// Sentinel for empty flags
    pub const NONE: RegFlags = 0;

    /// Basic (obsolete) regex
    pub const BASIC: RegFlags = tre::REG_BASIC;

    /// Extended POSIX regex
    pub const EXTENDED: RegFlags = tre::REG_EXTENDED;

    /// Case-insensitive matches
    pub const ICASE: RegFlags = tre::REG_ICASE;

    /// Interpret regex literally (all characters are non-special); aka `REG_NOSPEC`
    pub const LITERAL: RegFlags = tre::REG_LITERAL;

    /// Same meaning as [`RegcompFlags::LITERAL`]
    pub const NOSPEC: RegFlags = tre::REG_NOSPEC;

    /// Newline-sensitive matching
    pub const NEWLINE: RegFlags = tre::REG_NEWLINE;

    /// Don't report what was matched; only that it matched.
    pub const NOSUB: RegFlags = tre::REG_NOSUB;

    /// Concatenation is right-associative
    pub const RIGHT_ASSOC: RegFlags = tre::REG_RIGHT_ASSOC;

    /// Repetition operators are non-greedy by default
    pub const UNGREEDY: RegFlags = tre::REG_UNGREEDY;

    /// Use raw bytes
    pub const USEBYTES: RegFlags = tre::REG_USEBYTES;

    /// Construct a new set of empty flags
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Add a flag
    #[must_use]
    #[inline]
    pub const fn add(&self, flag: RegFlags) -> Self {
        Self(self.0 | flag)
    }

    /// Remove a flag
    #[must_use]
    #[inline]
    pub const fn remove(&self, flag: RegFlags) -> Self {
        Self(self.0 & !flag)
    }

    /// Get set flags as a [`RegFlags`].
    #[must_use]
    #[inline]
    pub const fn get(&self) -> RegFlags {
        self.0
    }
}

/// Flags to pass to [`regexec`](crate::regexec).
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegexecFlags(RegFlags);

impl RegexecFlags {
    /// Sentinel for empty flags
    pub const NONE: RegFlags = 0;

    /// Use the approximate matcher
    pub const APPROX_MATCHER: RegFlags = tre::REG_APPROX_MATCHER;

    /// Use the backtracking matcher
    pub const BACKTRACKING_MATCHER: RegFlags = tre::REG_BACKTRACKING_MATCHER;

    /// First character of the string is not the beginning of the line
    pub const NOTBOL: RegFlags = tre::REG_NOTBOL;

    /// Last character of the string is not the end of the line
    pub const NOTEOL: RegFlags = tre::REG_NOTEOL;

    /// Construct a new set of empty flags
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Add a flag
    #[must_use]
    #[inline]
    pub const fn add(&self, flag: RegFlags) -> Self {
        Self(self.0 | flag)
    }

    /// Remove a flag
    #[must_use]
    #[inline]
    pub const fn remove(&self, flag: RegFlags) -> Self {
        Self(self.0 & !flag)
    }

    /// Get set flags as a [`RegFlags`].
    #[must_use]
    #[inline]
    pub const fn get(&self) -> RegFlags {
        self.0
    }
}
