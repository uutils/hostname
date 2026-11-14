// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use uucore::error::UResult;

use crate::errors::DomainNameError;
use crate::net::set_domain_name;
use crate::utils::parse_domain_name_file;

pub(crate) fn from_file(path: &Path) -> UResult<()> {
    let domain_name = parse_domain_name_file(path)?;
    let domain_name =
        std::str::from_utf8(&domain_name).map_err(|_r| DomainNameError::InvalidDomainName)?;
    run(domain_name.encode_utf16().collect())
}

pub(crate) fn from_argument(domain_name: &OsStr) -> UResult<()> {
    run(domain_name.encode_wide().collect())
}

fn run(mut domain_name: Vec<u16>) -> UResult<()> {
    // Trim white space.
    while domain_name.first().is_some_and(u16_is_ascii_whitespace) {
        domain_name.remove(0);
    }

    while domain_name.last().is_some_and(u16_is_ascii_whitespace) {
        domain_name.pop();
    }

    validate_domain_name(&domain_name)?;

    domain_name.push(0); // Null-terminate.
    set_domain_name(&domain_name)
}

fn u16_is_ascii_whitespace(ch: &u16) -> bool {
    u8::try_from(*ch).is_ok_and(|b| b.is_ascii_whitespace())
}

fn u16_is_ascii_alphanumeric(ch: &u16) -> bool {
    u8::try_from(*ch).is_ok_and(|b| b.is_ascii_alphanumeric())
}

fn validate_domain_name(domain_name: &[u16]) -> Result<(), DomainNameError> {
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

    let (Some(first_byte), Some(last_byte)) = (domain_name.first(), domain_name.last()) else {
        return Err(DomainNameError::InvalidDomainName); // Empty name.
    };

    let is_disallowed_byte = move |ch: &u16| {
        !u16_is_ascii_alphanumeric(ch) && *ch != (b'-' as u16) && *ch != (b'.' as u16)
    };
    let is_disallowed_seq = move |seq: &[u16]| seq == DOT_DOT || seq == DOT_DASH || seq == DASH_DOT;

    if !u16_is_ascii_alphanumeric(first_byte)
        || !u16_is_ascii_alphanumeric(last_byte)
        || domain_name.iter().any(is_disallowed_byte)
        || domain_name.windows(2).any(is_disallowed_seq)
    {
        Err(DomainNameError::InvalidDomainName)
    } else {
        Ok(())
    }
}
