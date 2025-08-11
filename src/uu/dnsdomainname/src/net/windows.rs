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
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Windows not yet supported for dnsdomainname",
    ))
}

// Windows-compatible dummy type that matches libc::addrinfo interface
#[repr(C)]
pub(crate) struct DummyAddrInfo {
    pub ai_flags: libc::c_int,
    pub ai_family: libc::c_int,
    pub ai_socktype: libc::c_int,
    pub ai_protocol: libc::c_int,
    pub ai_addrlen: libc::size_t,
    pub ai_canonname: *mut libc::c_char,
    pub ai_addr: *mut libc::sockaddr,
    pub ai_next: *mut DummyAddrInfo,
}

static DUMMY_ADDR_INFO: DummyAddrInfo = DummyAddrInfo {
    ai_flags: 0,
    ai_family: 0,
    ai_socktype: 0,
    ai_protocol: 0,
    ai_addrlen: 0,
    ai_canonname: std::ptr::null_mut(),
    ai_addr: std::ptr::null_mut(),
    ai_next: std::ptr::null_mut(),
};

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

    pub(crate) fn first(&self) -> &crate::net::PlatformAddrInfo {
        // This should never be called on Windows due to the error above
        &DUMMY_ADDR_INFO
    }
}
