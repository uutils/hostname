// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod net;
mod print;

use crate::print::PrintHostName;
use clap::{Arg, ArgAction, Command, crate_version};
use std::io;
use uucore::{error::UResult, format_usage};

const ABOUT: &str = "Display the DNS domain name.";
const USAGE: &str = "dnsdomainname [-v]";

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let _net_lib_guard = net::LibraryGuard::load()?;

    let _verbose = matches.get_flag("verbose");
    // Note: verbose flag is accepted but doesn't change behavior in GNU implementation

    let domain_printer = print::DomainHostName;
    let mut stdout = io::stdout();

    domain_printer.print_host_name(&mut stdout)
}

#[must_use]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Be verbose and tell what's going on"),
        )
}
