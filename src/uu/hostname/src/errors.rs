// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fmt;

use uucore::error::UError;

#[derive(Debug, PartialEq, Eq)]
pub enum HostNameError {
    InvalidHostName,
    HostNameTooLong,
    NoLocalDomainName,
    SetHostNameDenied,
    #[cfg(not(target_family = "windows"))]
    GetNameOrAddrInfo(GetNameOrAddrInfoError),
}

impl fmt::Display for HostNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHostName => write!(f, "the specified hostname is invalid"),
            Self::HostNameTooLong => write!(f, "name too long"),
            Self::NoLocalDomainName => write!(f, "local domain name not set"),
            Self::SetHostNameDenied => write!(f, "you must be root to change the host name"),
            #[cfg(not(target_family = "windows"))]
            Self::GetNameOrAddrInfo(r) => write!(f, "{r}"),
        }
    }
}

impl UError for HostNameError {
    fn code(&self) -> i32 {
        1
    }

    fn usage(&self) -> bool {
        false
    }
}

impl std::error::Error for HostNameError {}

#[cfg(not(target_family = "windows"))]
#[derive(Debug, PartialEq, Eq)]
pub struct GetNameOrAddrInfoError(pub(crate) std::ffi::c_int);

#[cfg(not(target_family = "windows"))]
impl fmt::Display for GetNameOrAddrInfoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = unsafe { libc::gai_strerror(self.0) };
        if message.is_null() {
            write!(f, "domain name resolution failed: error {}", self.0)
        } else {
            let message = unsafe { std::ffi::CStr::from_ptr(message.cast()) };
            if let Ok(message) = message.to_str() {
                write!(f, "{message}")
            } else {
                write!(f, "{message:?}")
            }
        }
    }
}

#[cfg(not(target_family = "windows"))]
impl std::error::Error for GetNameOrAddrInfoError {}
