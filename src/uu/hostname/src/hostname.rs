// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::OsString;

use clap::{crate_version, Arg, ArgAction, ArgGroup, ArgMatches, Command};

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

/// Retrieve and display the host name.
fn query_host_name(args: ArgMatches) -> UResult<()> {
    if args.args_present() {
        todo!()
    }

    let hostname = hostname::get().unwrap_or(OsString::from(""));
    let hostname = hostname.to_string_lossy();
    println!("{}", hostname);
    Ok(())
}

fn set_host_name(_args: ArgMatches) -> UResult<()> {
    todo!()
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    match (matches.args_present(), matches.contains_id("set-group")) {
        (false, _) | (true, false) => query_host_name(matches),
        (true, true) => set_host_name(matches),
    }
}

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
            Arg::new(options::YP)
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
                .action(ArgAction::Set)
                .conflicts_with(options::HOSTNAME)
                .help("read host name or NIS domain name from given file"),
        )
        .arg(Arg::new(options::HOSTNAME).conflicts_with(options::FILE))
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
                    options::YP,
                ])
                .multiple(true)
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
