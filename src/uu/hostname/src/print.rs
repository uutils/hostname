// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(not(target_family = "windows"))]
mod unix;
#[cfg(target_family = "windows")]
mod windows;

use uucore::error::UResult;

pub(crate) trait PrintHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()>;
}

pub(crate) struct DefaultHostName;
pub(crate) struct AliasHostName;
pub(crate) struct DomainHostName;
pub(crate) struct FqdnHostName;
pub(crate) struct AllFqdnHostName;
pub(crate) struct IpAddressHostName;
pub(crate) struct AllIpAddressesHostName;
pub(crate) struct ShortHostName;
pub(crate) struct NisHostName;
