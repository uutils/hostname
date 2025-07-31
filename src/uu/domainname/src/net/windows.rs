// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::{OsString, c_int};
use std::os::windows::ffi::OsStringExt;

use uucore::error::UResult;
use windows_sys::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_INVALID_NAME, ERROR_INVALID_PARAMETER, ERROR_MORE_DATA,
};

use windows_sys::Win32::Networking::WinSock::WSAENAMETOOLONG;

use windows_sys::Win32::System::SystemInformation::{
    ComputerNamePhysicalDnsDomain, GetComputerNameExW, SetComputerNameExW,
};

use crate::errors::DomainNameError;

fn get_computer_name_ex_w(kind: c_int) -> std::io::Result<OsString> {
    use std::io::Error;

    let mut buffer: Vec<u16> = Vec::with_capacity(256);
    loop {
        let Ok(mut buffer_capacity) = u32::try_from(buffer.capacity()) else {
            break Err(Error::from_raw_os_error(WSAENAMETOOLONG));
        };

        let buffer_ptr = buffer.spare_capacity_mut().as_mut_ptr().cast();

        if unsafe { GetComputerNameExW(kind, buffer_ptr, &mut buffer_capacity) } == 0 {
            let err = Error::last_os_error();
            if err.raw_os_error() == Some(ERROR_MORE_DATA as i32) {
                // An error happened because a bigger buffer is needed.
                buffer.reserve((buffer_capacity as usize).saturating_sub(buffer.capacity()));
            } else {
                break Err(err);
            }
        } else {
            unsafe { buffer.set_len(buffer_capacity as usize) };
            break Ok(OsString::from_wide(&buffer));
        }
    }
}

pub(crate) fn domain_name() -> std::io::Result<Option<OsString>> {
    // TODO: Should we specify ComputerNameDnsDomain instead?
    // TODO: Should we call NetGetJoinInformation() instead?
    let name = get_computer_name_ex_w(ComputerNamePhysicalDnsDomain)?;
    Ok((!name.is_empty()).then_some(name))
}

pub(crate) fn set_domain_name(domain_name: &[u16]) -> UResult<()> {
    if unsafe { SetComputerNameExW(ComputerNamePhysicalDnsDomain, domain_name.as_ptr()) } != 0 {
        return Ok(());
    }

    let err = std::io::Error::last_os_error();
    match err.raw_os_error().map(|n| n as u32) {
        Some(ERROR_ACCESS_DENIED) => Err(Box::new(DomainNameError::SetDomainNameDenied)),

        Some(ERROR_INVALID_PARAMETER | ERROR_INVALID_NAME) => {
            Err(Box::new(DomainNameError::DomainNameTooLong))
        }

        _ => Err(err.into()),
    }
}
