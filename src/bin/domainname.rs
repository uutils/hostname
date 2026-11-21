// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Shared wrapper binary for domainname, dnsdomainname, nisdomainname, and
// ypdomainname. Which `hostname` flag to inject is resolved from argv[0],
// the same way src/bin/hostname.rs resolves a util name from the binary path.

use std::ffi::OsString;
use std::path::Path;
use std::process;

fn flag_for(binary_name: &str) -> &'static str {
    // Allow a PROG_PREFIX (e.g. "uu-"), matched like in src/bin/hostname.rs.
    if binary_name
        .strip_suffix("dnsdomainname")
        .is_some_and(|prefix| !prefix.ends_with(char::is_alphanumeric))
    {
        "-d"
    } else {
        "-y" // domainname, nisdomainname, ypdomainname
    }
}

fn main() {
    uucore::panic::mute_sigpipe_panic();

    let mut args = std::env::args_os();
    let binary = args.next().unwrap_or_default();
    let binary_name = Path::new(&binary)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("domainname");

    let iter = [
        OsString::from("hostname"),
        OsString::from(flag_for(binary_name)),
    ]
    .into_iter()
    .chain(args);

    process::exit(hostname::uumain(iter));
}

#[cfg(test)]
mod tests {
    use super::flag_for;

    #[test]
    fn test_flag_for() {
        for (binary_name, flag) in [
            ("dnsdomainname", "-d"),
            ("uu-dnsdomainname", "-d"),
            ("uu_dnsdomainname", "-d"),
            ("xdnsdomainname", "-y"),
            ("domainname", "-y"),
            ("uu-domainname", "-y"),
            ("nisdomainname", "-y"),
            ("ypdomainname", "-y"),
        ] {
            assert_eq!(flag_for(binary_name), flag, "binary name: {binary_name}");
        }
    }
}
