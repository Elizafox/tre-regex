// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

#[cfg(feature = "approx")]
mod approx;
mod comp;
mod exec;

#[cfg(feature = "approx")]
pub use crate::wchar::approx::*;
pub use crate::wchar::comp::*;
pub use crate::wchar::exec::*;
