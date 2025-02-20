// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::borrow::Cow;
use std::ffi::{CString, OsStr};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use uucore::error::UResult;

use crate::errors::HostNameError;
use crate::utils::{bytes_from_os_str, set_host_name};

pub(crate) fn from_file(path: &Path) -> UResult<()> {
    parse_host_name_file(path).map(Cow::Owned).and_then(run)
}

pub(crate) fn from_argument(host_name: &OsStr) -> UResult<()> {
    run(bytes_from_os_str(host_name))
}

fn run(mut host_name: Cow<[u8]>) -> UResult<()> {
    // Trim white space.
    match &mut host_name {
        Cow::Borrowed(name) => *name = name.trim_ascii(),

        Cow::Owned(name) => {
            while name.first().is_some_and(u8::is_ascii_whitespace) {
                name.remove(0);
            }

            while name.last().is_some_and(u8::is_ascii_whitespace) {
                name.pop();
            }
        }
    };

    let host_name = validate_host_name(host_name)?;

    set_host_name(&host_name)
}

fn parse_host_name_file(path: &Path) -> UResult<Vec<u8>> {
    let mut file = std::fs::File::open(path).map(BufReader::new)?;

    let first_byte = loop {
        let mut first_byte = [0_u8; 1];
        if let Err(err) = file.read_exact(&mut first_byte) {
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(Vec::default()); // Empty name.
            }
        }

        match first_byte[0] {
            b'\r' | b'\n' => {} // Empty line. Skip.

            b'#' => {
                file.skip_until(b'\n')?; // Comment line. Skip.
            }

            first_byte => break first_byte,
        }
    };

    let mut buffer = Vec::with_capacity(256);
    buffer.push(first_byte);
    file.read_until(b'\n', &mut buffer)?;
    while matches!(buffer.last().copied(), Some(b'\r' | b'\n')) {
        buffer.pop();
    }
    Ok(buffer)
}

fn validate_host_name(host_name: Cow<[u8]>) -> Result<CString, HostNameError> {
    // Rules:
    // - The only allowed prefix and suffix characters are alphanumeric.
    // - The only allowed characters inside are alphanumeric, '-' and '.'.
    // - The following sequences are disallowed: "..", ".-" and "-.".
    //
    // Reference: RFC 1035: Domain Names - Implementation And Specification,
    // section 2.3.1. Preferred name syntax.

    let (Some(first_byte), Some(last_byte)) = (host_name.first(), host_name.last()) else {
        return Err(HostNameError::InvalidHostName); // Empty name.
    };

    let is_disallowed_byte = move |b: &u8| !b.is_ascii_alphanumeric() && *b != b'-' && *b != b'.';
    let is_disallowed_seq = move |seq: &[u8]| seq == b".." || seq == b".-" || seq == b"-.";

    if !first_byte.is_ascii_alphanumeric()
        || !last_byte.is_ascii_alphanumeric()
        || host_name.iter().any(is_disallowed_byte)
        || host_name.windows(2).any(is_disallowed_seq)
    {
        return Err(HostNameError::InvalidHostName);
    }

    let mut host_name = host_name.into_owned();
    host_name.push(0_u8);
    Ok(unsafe { CString::from_vec_with_nul_unchecked(host_name) })
}
