// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use libc::{AF_UNSPEC, AI_CANONNAME, SOCK_DGRAM};
use std::ffi::CStr;
use uucore::error::UResult;

use crate::net::{AddressInfo, host_name};
use crate::print::{DomainHostName, PrintHostName};

impl PrintHostName for DomainHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let address_info = AddressInfo::new(&host_name()?, AF_UNSPEC, SOCK_DGRAM, 0, AI_CANONNAME)?;

        let canonical_name = address_info.first().ai_canonname;
        if canonical_name.is_null() {
            return Ok(()); // No canonical name set.
        };

        let Some(domain_name) = unsafe { CStr::from_ptr(canonical_name) }
            .to_bytes()
            .splitn(2, |&byte| byte == b'.')
            .nth(1)
        else {
            return Ok(()); // Canonical name contains zero dots.
        };

        out.write_all(domain_name)?;
        out.write_all(b"\n").map_err(From::from)
    }
}
