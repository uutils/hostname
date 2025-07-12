#[cfg(not(target_family = "windows"))]
mod unix;


#[cfg(not(target_family = "windows"))]
pub(crate) use unix::*;


pub(crate) struct LibraryGuard;
