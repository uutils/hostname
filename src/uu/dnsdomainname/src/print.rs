#[cfg(not(target_family = "windows"))]
mod unix;
#[cfg(target_family = "windows")]
mod windows;

use uucore::error::UResult;

pub(crate) trait PrintHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()>;
}

pub(crate) struct DomainHostName;
