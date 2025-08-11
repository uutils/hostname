#[cfg(not(target_family = "windows"))]
mod unix;
#[cfg(target_family = "windows")]
mod windows;

#[cfg(not(target_family = "windows"))]
pub(crate) use unix::*;
#[cfg(target_family = "windows")]
pub(crate) use windows::*;

pub(crate) struct LibraryGuard;

// Type alias to make the interface consistent across platforms
#[cfg(not(target_family = "windows"))]
pub(crate) type PlatformAddrInfo = libc::addrinfo;
#[cfg(target_family = "windows")]
pub(crate) type PlatformAddrInfo = windows::DummyAddrInfo;
