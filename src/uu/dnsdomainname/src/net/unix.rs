// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::{CStr, CString, c_int};
use std::ptr;
use std::ptr::NonNull;

impl crate::net::LibraryGuard {
    pub(crate) fn load() -> std::io::Result<Self> {
        Ok(Self)
    }
}

fn max_host_name_size() -> usize {
    const _POSIX_HOST_NAME_MAX: usize = 255;

    usize::try_from(unsafe { libc::sysconf(libc::_SC_HOST_NAME_MAX) })
        .unwrap_or(_POSIX_HOST_NAME_MAX)
        .max(_POSIX_HOST_NAME_MAX)
        .saturating_add(1)
}

pub(crate) fn host_name() -> std::io::Result<CString> {
    let mut buffer: Vec<u8> = vec![0_u8; max_host_name_size()];
    loop {
        errno::set_errno(errno::Errno(0));

        if unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } == -1 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ENAMETOOLONG) {
                break Err(err);
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

#[repr(transparent)]
pub(crate) struct AddressInfo(NonNull<libc::addrinfo>);

impl AddressInfo {
    pub(crate) fn new(
        host_name: &CStr,
        hint_family: c_int,
        hint_socktype: c_int,
        hint_protocol: c_int,
        hint_flags: c_int,
    ) -> std::io::Result<Self> {
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
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))
        } else {
            Err(std::io::Error::from_raw_os_error(r))
        }
    }

    pub(crate) fn first(&self) -> &libc::addrinfo {
        unsafe { self.0.as_ref() }
    }
}

impl Drop for AddressInfo {
    fn drop(&mut self) {
        unsafe { libc::freeaddrinfo(self.0.as_ptr()) }
    }
}
