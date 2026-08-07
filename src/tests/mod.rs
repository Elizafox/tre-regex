// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2026 Elizabeth Kiara Regina Ashford

#[cfg(feature = "approx")]
mod approx;
mod comp;
mod err;
mod exec;
mod flags;
mod regex;
#[cfg(feature = "wchar")]
mod wchar;
