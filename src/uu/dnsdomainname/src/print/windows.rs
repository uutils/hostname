// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uucore::error::UResult;

use crate::print::{DomainHostName, PrintHostName};

impl PrintHostName for DomainHostName {
    fn print_host_name(&self, _out: &mut dyn std::io::Write) -> UResult<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows not yet supported for dnsdomainname",
        )
        .into())
    }
}
