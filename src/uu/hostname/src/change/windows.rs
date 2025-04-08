// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use uucore::error::UResult;

use crate::errors::HostNameError;
use crate::net::set_host_name;
use crate::utils::parse_host_name_file;

pub(crate) fn from_file(path: &Path) -> UResult<()> {
    let host_name = parse_host_name_file(path)?;
    let host_name = std::str::from_utf8(&host_name).map_err(|_r| HostNameError::InvalidHostName)?;
    run(host_name.encode_utf16().collect())
}

pub(crate) fn from_argument(host_name: &OsStr) -> UResult<()> {
    run(host_name.encode_wide().collect())
}

fn run(mut host_name: Vec<u16>) -> UResult<()> {
    // Trim white space.
    while host_name.first().is_some_and(u16_is_ascii_whitespace) {
        host_name.remove(0);
    }

    while host_name.last().is_some_and(u16_is_ascii_whitespace) {
        host_name.pop();
    }

    validate_host_name(&host_name)?;

    host_name.push(0); // Null-terminate.
    set_host_name(&host_name)
}

fn u16_is_ascii_whitespace(ch: &u16) -> bool {
    u8::try_from(*ch).is_ok_and(|b| b.is_ascii_whitespace())
}

fn u16_is_ascii_alphanumeric(ch: &u16) -> bool {
    u8::try_from(*ch).is_ok_and(|b| b.is_ascii_alphanumeric())
}

fn validate_host_name(host_name: &[u16]) -> Result<(), HostNameError> {
    // Rules:
    // - The only allowed prefix and suffix characters are alphanumeric.
    // - The only allowed characters inside are alphanumeric, '-' and '.'.
    // - The following sequences are disallowed: "..", ".-" and "-.".
    //
    // Reference: RFC 1035: Domain Names - Implementation And Specification,
    // section 2.3.1. Preferred name syntax.

    const DOT_DOT: [u16; 2] = [b'.' as u16, b'.' as u16];
    const DOT_DASH: [u16; 2] = [b'.' as u16, b'-' as u16];
    const DASH_DOT: [u16; 2] = [b'-' as u16, b'.' as u16];

    let (Some(first_byte), Some(last_byte)) = (host_name.first(), host_name.last()) else {
        return Err(HostNameError::InvalidHostName); // Empty name.
    };

    let is_disallowed_byte = move |ch: &u16| {
        !u16_is_ascii_alphanumeric(ch) && *ch != (b'-' as u16) && *ch != (b'.' as u16)
    };
    let is_disallowed_seq = move |seq: &[u16]| seq == DOT_DOT || seq == DOT_DASH || seq == DASH_DOT;

    if !u16_is_ascii_alphanumeric(first_byte)
        || !u16_is_ascii_alphanumeric(last_byte)
        || host_name.iter().any(is_disallowed_byte)
        || host_name.windows(2).any(is_disallowed_seq)
    {
        Err(HostNameError::InvalidHostName)
    } else {
        Ok(())
    }
}
