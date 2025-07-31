// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fmt;

use uucore::error::UError;

#[derive(Debug, PartialEq, Eq)]
pub enum DomainNameError {
    InvalidDomainName,
    DomainNameTooLong,
    NoLocalDomainName,
    SetDomainNameDenied,
}

impl fmt::Display for DomainNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDomainName => write!(f, "the specified domainname is invalid"),
            Self::DomainNameTooLong => write!(f, "name too long"),
            Self::NoLocalDomainName => write!(f, "local domain name not set"),
            Self::SetDomainNameDenied => write!(f, "you must be root to change the domain name"),
        }
    }
}

impl UError for DomainNameError {
    fn code(&self) -> i32 {
        1
    }

    fn usage(&self) -> bool {
        false
    }
}

impl std::error::Error for DomainNameError {}

#[cfg(not(target_family = "windows"))]
#[derive(Debug, PartialEq, Eq)]
pub struct GetNameOrAddrInfoError(pub(crate) std::ffi::c_int);
