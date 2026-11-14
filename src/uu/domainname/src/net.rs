#[cfg(not(target_family = "windows"))]
mod unix;
#[cfg(target_family = "windows")]
mod windows;

#[cfg(not(target_family = "windows"))]
pub(crate) use unix::*;
#[cfg(target_family = "windows")]
pub(crate) use windows::*;
