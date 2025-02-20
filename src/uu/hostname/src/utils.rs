// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::borrow::Cow;
use std::ffi::{c_int, CStr, CString, OsStr};
use std::ptr;
use std::ptr::NonNull;

use uucore::error::UResult;

use crate::errors::{GetNameOrAddrInfoError, HostNameError};

pub(crate) fn bytes_from_os_str(os_str: &OsStr) -> Cow<[u8]> {
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::ffi::OsStrExt;
        Cow::Borrowed(os_str.as_bytes())
    }

    #[cfg(target_family = "wasm")]
    {
        use std::os::wasm::ffi::OsStrExt;
        Cow::Borrowed(os_str.as_bytes())
    }

    #[cfg(target_family = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        Cow::Owned(os_str.encode_wide().flat_map(u16::to_le_bytes).collect())
    }
}

pub(crate) fn max_host_name_size() -> usize {
    const _POSIX_HOST_NAME_MAX: usize = 255;

    usize::try_from(unsafe { libc::sysconf(libc::_SC_HOST_NAME_MAX) })
        .unwrap_or(_POSIX_HOST_NAME_MAX)
        .max(_POSIX_HOST_NAME_MAX)
        .saturating_add(1)
}

pub(crate) fn host_name() -> UResult<CString> {
    let mut buffer: Vec<u8> = vec![0_u8; max_host_name_size()];
    loop {
        errno::set_errno(errno::Errno(0));

        if unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } == -1 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ENAMETOOLONG) {
                break Err(err.into());
            }
            // else an error happened because a bigger buffer is needed.
        } else if let Some(index) = buffer.iter().position(|&b| b == 0_u8) {
            buffer.truncate(index + 1);
            break Ok(unsafe { CString::from_vec_with_nul_unchecked(buffer) });
        }
        // else truncation happened because a bigger buffer is needed.

        buffer.resize_with(buffer.len() + 4096, Default::default);
    }
}

pub(crate) fn domain_name() -> UResult<CString> {
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "solaris",
    ))]
    type CBufferLength = c_int;

    #[cfg(not(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "solaris",
    )))]
    type CBufferLength = usize;

    let mut buffer: Vec<u8> = vec![0_u8; 256];
    loop {
        #[allow(irrefutable_let_patterns)]
        let Ok(buffer_len) = CBufferLength::try_from(buffer.len()) else {
            break Err(Box::new(HostNameError::HostNameTooLong));
        };

        errno::set_errno(errno::Errno(0));

        if unsafe { libc::getdomainname(buffer.as_mut_ptr().cast(), buffer_len) } == -1 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ENAMETOOLONG) {
                break Err(err.into());
            }
            // else an error happened because a bigger buffer is needed.
        } else if let Some(index) = buffer.iter().position(|&b| b == 0_u8) {
            buffer.truncate(index + 1);
            break Ok(unsafe { CString::from_vec_with_nul_unchecked(buffer) });
        }
        // else truncation happened because a bigger buffer is needed.

        buffer.resize_with(buffer.len() + 4096, Default::default);
    }
}

pub(crate) fn short_host_name() -> UResult<CString> {
    let mut bytes = host_name()?.into_bytes_with_nul();
    if let Some(index) = bytes.iter().position(|&byte| byte == b'.') {
        bytes.truncate(index);
        bytes.push(0_u8);
    }

    Ok(unsafe { CString::from_vec_with_nul_unchecked(bytes) })
}

pub(crate) fn set_host_name(host_name: &CStr) -> UResult<()> {
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "solaris",
    ))]
    type CHostNameLength = c_int;

    #[cfg(not(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "solaris",
    )))]
    type CHostNameLength = usize;

    #[allow(irrefutable_let_patterns)]
    let Ok(host_name_len) = CHostNameLength::try_from(host_name.count_bytes()) else {
        return Err(Box::new(HostNameError::HostNameTooLong));
    };

    if unsafe { libc::sethostname(host_name.as_ptr(), host_name_len) } == -1 {
        let err = std::io::Error::last_os_error();
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => Err(Box::new(HostNameError::SetHostNameDenied)),
            std::io::ErrorKind::InvalidInput => Err(Box::new(HostNameError::HostNameTooLong)),
            _ => Err(err.into()),
        }
    } else {
        Ok(())
    }
}

#[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
pub(crate) fn get_name_info(
    address: *const libc::sockaddr,
    address_size: libc::socklen_t,
    flags: c_int,
) -> Result<CString, HostNameError> {
    #[cfg(any(
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd"
    ))]
    type CBufferLength = libc::socklen_t;

    #[cfg(not(any(
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd"
    )))]
    type CBufferLength = usize;

    #[allow(clippy::unnecessary_cast)]
    let initial_size = max_host_name_size().min(CBufferLength::MAX as usize);
    let mut buffer: Vec<u8> = vec![0_u8; initial_size];

    loop {
        #[allow(irrefutable_let_patterns)]
        let Ok(buffer_len) = CBufferLength::try_from(buffer.len()) else {
            return Err(HostNameError::HostNameTooLong);
        };

        let r = unsafe {
            libc::getnameinfo(
                address,
                address_size,
                buffer.as_mut_ptr().cast(),
                buffer_len,
                ptr::null_mut(),
                0,
                flags,
            )
        };

        match r {
            libc::EAI_OVERFLOW => {}

            0 => {
                if let Some(index) = buffer.iter().position(|&byte| byte == 0_u8) {
                    buffer.truncate(index + 1);
                    break Ok(unsafe { CString::from_vec_with_nul_unchecked(buffer) });
                }
            }

            _ => break Err(HostNameError::GetNameOrAddrInfo(GetNameOrAddrInfoError(r))),
        }

        buffer.resize_with(buffer.len() + 4096, Default::default);
    }
}

#[repr(transparent)]
pub(crate) struct InterfaceAddresses(NonNull<libc::ifaddrs>);

impl InterfaceAddresses {
    pub(crate) fn new() -> UResult<Self> {
        let mut ptr: *mut libc::ifaddrs = ptr::null_mut();

        if unsafe { libc::getifaddrs(&mut ptr) } == -1 {
            Err(std::io::Error::last_os_error().into())
        } else {
            NonNull::new(ptr)
                .map(Self)
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData).into())
        }
    }

    pub(crate) fn iter(&self) -> InterfaceAddressesIter {
        InterfaceAddressesIter {
            _ia: self,
            ptr: Some(self.0),
        }
    }
}

impl Drop for InterfaceAddresses {
    fn drop(&mut self) {
        unsafe {
            libc::freeifaddrs(self.0.as_ptr());
        }
    }
}

pub(crate) struct InterfaceAddressesIter<'ia> {
    _ia: &'ia InterfaceAddresses,
    ptr: Option<NonNull<libc::ifaddrs>>,
}

impl<'ia> Iterator for InterfaceAddressesIter<'ia> {
    type Item = &'ia libc::ifaddrs;

    fn next(&mut self) -> Option<Self::Item> {
        let element = unsafe { self.ptr?.as_ref() };
        self.ptr = NonNull::new(element.ifa_next);
        Some(element)
    }
}

#[repr(transparent)]
pub(crate) struct AddressInfo(NonNull<libc::addrinfo>);

impl AddressInfo {
    pub(crate) fn new(
        host_name: &CStr,
        hint_family: c_int,
        hint_socktype: c_int,
        hint_protocol: c_int,
        hint_flags: c_int,
    ) -> UResult<Self> {
        let mut c_hints: libc::addrinfo = unsafe { std::mem::zeroed() };
        c_hints.ai_family = hint_family;
        c_hints.ai_socktype = hint_socktype;
        c_hints.ai_protocol = hint_protocol;
        c_hints.ai_flags = hint_flags;

        let mut ptr: *mut libc::addrinfo = ptr::null_mut();

        let r = unsafe { libc::getaddrinfo(host_name.as_ptr(), ptr::null(), &c_hints, &mut ptr) };
        if r == 0 {
            NonNull::new(ptr)
                .map(Self)
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData).into())
        } else {
            Err(Box::new(HostNameError::GetNameOrAddrInfo(
                GetNameOrAddrInfoError(r),
            )))
        }
    }

    pub(crate) fn first(&self) -> &libc::addrinfo {
        unsafe { self.0.as_ref() }
    }

    pub(crate) fn iter(&self) -> AddressInfoIter {
        AddressInfoIter {
            _ia: self,
            ptr: Some(self.0),
        }
    }
}

impl Drop for AddressInfo {
    fn drop(&mut self) {
        unsafe {
            libc::freeaddrinfo(self.0.as_ptr());
        }
    }
}

pub(crate) struct AddressInfoIter<'ia> {
    _ia: &'ia AddressInfo,
    ptr: Option<NonNull<libc::addrinfo>>,
}

impl<'ia> Iterator for AddressInfoIter<'ia> {
    type Item = &'ia libc::addrinfo;

    fn next(&mut self) -> Option<Self::Item> {
        let element = unsafe { self.ptr?.as_ref() };
        self.ptr = NonNull::new(element.ai_next);
        Some(element)
    }
}

fn in6_is_addr_multicast(addr: &libc::in6_addr) -> bool {
    addr.s6_addr[0] == 0xff
}

pub(crate) fn in6_is_addr_linklocal(addr: &libc::in6_addr) -> bool {
    addr.s6_addr[0] == 0xfe && ((addr.s6_addr[1] & 0xc0) == 0x80)
}

pub(crate) fn in6_is_addr_mc_linklocal(addr: &libc::in6_addr) -> bool {
    in6_is_addr_multicast(addr) && ((addr.s6_addr[1] & 0xf) == 0x2)
}
