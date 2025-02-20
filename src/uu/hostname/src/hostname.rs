// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod change;
mod errors;
mod print;
mod utils;

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{crate_version, value_parser, Arg, ArgAction, ArgGroup, Command};
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("hostname.md");
const USAGE: &str = help_usage!("hostname.md");

pub mod options {
    pub static ALIAS: &str = "alias";
    pub static ALL_FQDNS: &str = "all-fqdns";
    pub static ALL_IP_ADDRESSES: &str = "all-ip-addresses";
    pub static BOOT: &str = "boot";
    pub static DOMAIN: &str = "domain";
    pub static FILE: &str = "file";
    pub static FILENAME: &str = "filename";
    pub static FQDN: &str = "fqdn";
    pub static HOSTNAME: &str = "hostname";
    pub static IP_ADDRESS: &str = "ip-address";
    pub static LONG: &str = "long";
    pub static NIS: &str = "nis";
    pub static SHORT: &str = "short";
    pub static YP: &str = "yp";
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let args = uu_app().try_get_matches_from(args)?;

    if args.contains_id("set-group") {
        if let Some(path) = args.get_one::<PathBuf>(options::FILE) {
            change::from_file(path)
        } else {
            let host_name = args
                .get_one::<OsString>(options::HOSTNAME)
                .expect("hostname must be specified");

            change::from_argument(host_name)
        }
    } else {
        let host_name: &mut dyn print::PrintHostName = if args.get_flag(options::ALIAS) {
            &mut print::AliasHostName
        } else if args.get_flag(options::DOMAIN) {
            &mut print::DomainHostName
        } else if args.get_flag(options::FQDN) {
            &mut print::FqdnHostName
        } else if args.get_flag(options::ALL_FQDNS) {
            &mut print::AllFqdnHostName
        } else if args.get_flag(options::IP_ADDRESS) {
            &mut print::IpAddressHostName
        } else if args.get_flag(options::ALL_IP_ADDRESSES) {
            &mut print::AllIpAddressesHostName
        } else if args.get_flag(options::SHORT) {
            &mut print::ShortHostName
        } else if args.get_flag(options::NIS) {
            &mut print::NisHostName
        } else {
            &mut print::DefaultHostName
        };

        let mut stdout = std::io::stdout();
        host_name.print_host_name(&mut stdout)
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
            Arg::new(options::ALIAS)
                .short('a')
                .long(options::ALIAS)
                .action(ArgAction::SetTrue)
                .help("alias names"),
        )
        .arg(
            Arg::new(options::DOMAIN)
                .short('d')
                .long(options::DOMAIN)
                .action(ArgAction::SetTrue)
                .help("DNS domain name"),
        )
        .arg(
            Arg::new(options::FQDN)
                .short('f')
                .long(options::FQDN)
                .visible_alias(options::LONG)
                .action(ArgAction::SetTrue)
                .help("long host name (FQDN)"),
        )
        .arg(
            Arg::new(options::ALL_FQDNS)
                .short('A')
                .long(options::ALL_FQDNS)
                .action(ArgAction::SetTrue)
                .help("all long host names (FQDNs)"),
        )
        .arg(
            Arg::new(options::IP_ADDRESS)
                .short('i')
                .long(options::IP_ADDRESS)
                .action(ArgAction::SetTrue)
                .help("addresses for the host name"),
        )
        .arg(
            Arg::new(options::ALL_IP_ADDRESSES)
                .short('I')
                .long(options::ALL_IP_ADDRESSES)
                .action(ArgAction::SetTrue)
                .help("all addresses for the host"),
        )
        .arg(
            Arg::new(options::SHORT)
                .short('s')
                .long(options::SHORT)
                .action(ArgAction::SetTrue)
                .help("short host name"),
        )
        .arg(
            Arg::new(options::NIS)
                .short('y')
                .long(options::YP)
                .visible_alias(options::NIS)
                .action(ArgAction::SetTrue)
                .help("NIS/YP domain name"),
        )
        .arg(
            Arg::new(options::BOOT)
                .short('b')
                .long(options::BOOT)
                .action(ArgAction::SetTrue)
                .help("set default hostname if none available"),
        )
        .arg(
            Arg::new(options::FILE)
                .short('F')
                .long(options::FILE)
                .value_name(options::FILENAME)
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .conflicts_with(options::HOSTNAME)
                .help("read host name or NIS domain name from given file"),
        )
        .arg(
            Arg::new(options::HOSTNAME)
                .value_parser(value_parser!(OsString))
                .conflicts_with(options::FILE),
        )
        .group(
            ArgGroup::new("get-group")
                .args([
                    options::ALIAS,
                    options::DOMAIN,
                    options::FQDN,
                    options::ALL_FQDNS,
                    options::IP_ADDRESS,
                    options::ALL_IP_ADDRESSES,
                    options::SHORT,
                    options::NIS,
                ])
                .multiple(false)
                .conflicts_with("set-group"),
        )
        .group(
            ArgGroup::new("set-group")
                .args([options::BOOT, options::FILE, options::HOSTNAME])
                .multiple(true)
                .requires("source-group")
                .conflicts_with("get-group"),
        )
        .group(
            ArgGroup::new("source-group")
                .args([options::FILE, options::HOSTNAME])
                .multiple(false),
        )
}
