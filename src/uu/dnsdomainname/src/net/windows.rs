// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::CString;

impl crate::net::LibraryGuard {
    pub(crate) fn load() -> std::io::Result<Self> {
        Ok(Self)
    }
}

pub(crate) fn host_name() -> std::io::Result<CString> {
    // Windows implementation would go here
    // For now, return an error since we're focusing on Unix
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Windows not yet supported for dnsdomainname",
    ))
}

pub(crate) struct AddressInfo;

impl AddressInfo {
    pub(crate) fn new(
        _host_name: &std::ffi::CStr,
        _hint_family: std::ffi::c_int,
        _hint_socktype: std::ffi::c_int,
        _hint_protocol: std::ffi::c_int,
        _hint_flags: std::ffi::c_int,
    ) -> std::io::Result<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows not yet supported for dnsdomainname",
        ))
    }

    pub(crate) fn first(&self) -> &libc::addrinfo {
        panic!("Windows not yet supported for dnsdomainname")
    }
}
