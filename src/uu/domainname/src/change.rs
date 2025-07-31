// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(not(target_family = "windows"))]
pub(crate) mod unix;
#[cfg(target_family = "windows")]
pub(crate) mod windows;

#[cfg(not(target_family = "windows"))]
pub(crate) use unix::{from_argument, from_file};
#[cfg(target_family = "windows")]
pub(crate) use windows::{from_argument, from_file};
