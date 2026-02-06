// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod change;
mod errors;
mod net;
mod print;
mod utils;

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgGroup, Command, crate_version, value_parser};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("domainname.md");
const USAGE: &str = help_usage!("domainname.md");

pub mod options {
    pub static FILE: &str = "file";
    pub static FILENAME: &str = "filename";
    pub static DOMAINNAME: &str = "domainname";
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let args = uu_app().try_get_matches_from(args)?;

    if args.contains_id("set-group") {
        if let Some(path) = args.get_one::<PathBuf>(options::FILE) {
            change::from_file(path)
        } else {
            let domain_name = args
                .get_one::<OsString>(options::DOMAINNAME)
                .expect("domainname must be specified");

            change::from_argument(domain_name)
        }
    } else {
        let mut stdout = std::io::stdout();
        print::print_domain_name(&mut stdout)
    }
}

#[must_use]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg(
            Arg::new(options::FILE)
                .short('F')
                .long(options::FILE)
                .value_name(options::FILENAME)
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .conflicts_with(options::DOMAINNAME)
                .help("read domain name from given file"),
        )
        .arg(
            Arg::new(options::DOMAINNAME)
                .value_parser(value_parser!(OsString))
                .conflicts_with(options::FILE),
        )
        .group(
            ArgGroup::new("set-group")
                .args([options::FILE, options::DOMAINNAME])
                .multiple(true)
                .requires("source-group"),
        )
        .group(
            ArgGroup::new("source-group")
                .args([options::FILE, options::DOMAINNAME])
                .multiple(false),
        )
}
