// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uucore::error::UResult;

use crate::errors::DomainNameError;
use crate::net::domain_name;

pub(crate) fn print_domain_name(out: &mut dyn std::io::Write) -> UResult<()> {
    if let Some(domain_name) = domain_name()? {
        out.write_all(domain_name.as_encoded_bytes())?;
        out.write_all(b"\n").map_err(From::from)
    } else {
        Err(Box::new(DomainNameError::NoLocalDomainName))
    }
}
