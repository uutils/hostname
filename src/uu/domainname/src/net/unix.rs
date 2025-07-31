// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::{CStr, CString};

use uucore::error::UResult;

use crate::errors::DomainNameError;

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "solaris",
))]
use std::ffi::c_int;

pub(crate) fn domain_name() -> UResult<Option<CString>> {
    let mut buffer: Vec<u8> = vec![0_u8; 256];
    loop {
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "illumos",
            target_os = "ios",
            target_os = "macos",
            target_os = "solaris",
        ))]
        let Ok(buffer_len) = c_int::try_from(buffer.len()) else {
            break Err(Box::new(DomainNameError::DomainNameTooLong));
        };

        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "illumos",
            target_os = "ios",
            target_os = "macos",
            target_os = "solaris",
        )))]
        let buffer_len = buffer.len();

        errno::set_errno(errno::Errno(0));

        if unsafe { libc::getdomainname(buffer.as_mut_ptr().cast(), buffer_len) } == -1 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ENAMETOOLONG) {
                break Err(err.into());
            }
            // else an error happened because a bigger buffer is needed.
        } else if let Some(index) = buffer.iter().position(|&b| b == 0_u8) {
            buffer.truncate(index + 1);

            break Ok(Some(unsafe {
                CString::from_vec_with_nul_unchecked(buffer)
            }));
        }
        // else truncation happened because a bigger buffer is needed.

        buffer.resize_with(buffer.len() + 4096, Default::default);
    }
}

pub(crate) fn set_domain_name(domain_name: &CStr) -> UResult<()> {
    use std::io::{Error, ErrorKind};

    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "solaris",
    ))]
    let Ok(domain_name_len) = c_int::try_from(domain_name.count_bytes()) else {
        return Err(Box::new(DomainNameError::DomainNameTooLong));
    };

    #[cfg(not(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "solaris",
    )))]
    let domain_name_len = domain_name.count_bytes();

    if unsafe { libc::setdomainname(domain_name.as_ptr(), domain_name_len) } != -1 {
        return Ok(());
    }

    let err = Error::last_os_error();
    match err.kind() {
        ErrorKind::PermissionDenied => Err(Box::new(DomainNameError::SetDomainNameDenied)),
        ErrorKind::InvalidInput => Err(Box::new(DomainNameError::DomainNameTooLong)),
        _ => Err(err.into()),
    }
}
