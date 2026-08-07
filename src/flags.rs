// SPDX-License-Identifier: BSD-2-Clause
// See LICENSE file in the project root for full license text.

use std::ffi::c_int;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::tre;

/// The integer type used by TRE for flags.
pub type RegFlags = c_int;

macro_rules! flags {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($(#[$flag_meta:meta])* $flag:ident = $value:expr;)*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name(RegFlags);

        impl $name {
            /// No flags.
            pub const NONE: Self = Self(0);

            $($(#[$flag_meta])* pub const $flag: Self = Self($value);)*

            /// Constructs an empty set of flags.
            #[must_use]
            pub const fn new() -> Self {
                Self::NONE
            }

            /// Constructs flags without discarding bits unknown to this crate.
            #[must_use]
            pub const fn from_bits_retain(bits: RegFlags) -> Self {
                Self(bits)
            }

            /// Returns the underlying TRE flags.
            #[must_use]
            pub const fn bits(self) -> RegFlags {
                self.0
            }

            /// Returns `true` when no flags are set.
            #[must_use]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            /// Returns `true` when all flags in `other` are set.
            #[must_use]
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            /// Returns `true` when any flags in `other` are set.
            #[must_use]
            pub const fn intersects(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }

            /// Returns these flags with `flag` added.
            #[must_use]
            pub const fn add(self, flag: Self) -> Self {
                Self(self.0 | flag.0)
            }

            /// Returns these flags with `flag` removed.
            #[must_use]
            pub const fn remove(self, flag: Self) -> Self {
                Self(self.0 & !flag.0)
            }
        }

        impl BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self { Self(self.0 | rhs.0) }
        }

        impl BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
        }

        impl BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self { Self(self.0 & rhs.0) }
        }

        impl BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) { self.0 &= rhs.0; }
        }

        impl Not for $name {
            type Output = Self;
            fn not(self) -> Self { Self(!self.0) }
        }
    };
}

flags! {
    /// Flags used when compiling a [`Regex`](crate::Regex).
    RegcompFlags {
        /// Basic (obsolete) regular expressions.
        BASIC = tre::REG_BASIC;
        /// Extended POSIX regular expressions.
        EXTENDED = tre::REG_EXTENDED;
        /// Case-insensitive matching.
        ICASE = tre::REG_ICASE;
        /// Interpret the expression literally.
        LITERAL = tre::REG_LITERAL;
        /// Alias for [`LITERAL`](Self::LITERAL).
        NOSPEC = tre::REG_NOSPEC;
        /// Newline-sensitive matching.
        NEWLINE = tre::REG_NEWLINE;
        /// Report only whether a match exists.
        NOSUB = tre::REG_NOSUB;
        /// Make concatenation right-associative.
        RIGHT_ASSOC = tre::REG_RIGHT_ASSOC;
        /// Make repetition operators non-greedy by default.
        UNGREEDY = tre::REG_UNGREEDY;
        /// Match raw bytes.
        USEBYTES = tre::REG_USEBYTES;
    }
}

flags! {
    /// Flags used when executing a compiled [`Regex`](crate::Regex).
    RegexecFlags {
        /// Use the approximate matcher.
        APPROX_MATCHER = tre::REG_APPROX_MATCHER;
        /// Use the backtracking matcher.
        BACKTRACKING_MATCHER = tre::REG_BACKTRACKING_MATCHER;
        /// Treat the first character as not being the beginning of a line.
        NOTBOL = tre::REG_NOTBOL;
        /// Treat the last character as not being the end of a line.
        NOTEOL = tre::REG_NOTEOL;
    }
}
