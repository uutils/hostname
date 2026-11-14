// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Tests for the domainname utilities: dnsdomainname, domainname, nisdomainname, and ypdomainname wrapper binaries,
// These binaries are built from src/bin/domainname.rs and delegates to hostname.

use assert_cmd::Command;
use predicates::prelude::*;

/// Run `bin` with `args` and return its stdout, asserting that it succeeded.
fn stdout_of(bin: &str, args: &[&str]) -> Vec<u8> {
    Command::cargo_bin(bin)
        .unwrap()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

/// Assert that `--help` succeeds and prints the same help text as `hostname --help`
fn assert_help(bin: &str) {
    let expected = stdout_of("hostname", &["--help"]);
    assert!(!expected.is_empty());
    Command::cargo_bin(bin)
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(expected);
}

/// Assert that `--version` succeeds and prints the binary name
fn assert_version(bin: &str) {
    Command::cargo_bin(bin)
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(bin));
}

#[test]
fn dnsdomainname_help() {
    assert_help("dnsdomainname");
}

#[test]
fn domainname_help() {
    assert_help("domainname");
}

#[test]
fn nisdomainname_help() {
    assert_help("nisdomainname");
}

#[test]
fn ypdomainname_help() {
    assert_help("ypdomainname");
}

#[test]
fn dnsdomainname_version() {
    assert_version("dnsdomainname");
}

#[test]
fn domainname_version() {
    assert_version("domainname");
}

#[test]
fn nisdomainname_version() {
    assert_version("nisdomainname");
}

#[test]
fn ypdomainname_version() {
    assert_version("ypdomainname");
}

/// Setting a name as root would actually change the domain of the machine running the tests.
#[cfg(unix)]
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// The NIS domain name as reported by getdomainname(2), or `None` when it is unset.
#[cfg(unix)]
fn nis_domain_name() -> Option<String> {
    let mut buffer = [0_u8; 256];
    let rc = unsafe { libc::getdomainname(buffer.as_mut_ptr().cast(), buffer.len() as _) };
    assert_eq!(rc, 0, "getdomainname failed");
    let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    let name = String::from_utf8(buffer[..len].to_vec()).unwrap();
    // Linux reports an unset domain name as "(none)".
    (name != "(none)").then_some(name)
}

/// Assert that `bin` prints the NIS domain name, or errors when none is set
#[cfg(unix)]
fn assert_prints_nis_domain(bin: &str) {
    let assert = Command::cargo_bin(bin).unwrap().assert();
    match nis_domain_name() {
        Some(name) => {
            assert.success().stdout(format!("{name}\n"));
        }
        None => {
            assert
                .failure()
                .code(1)
                .stderr(predicate::str::contains("local domain name not set"));
        }
    }
}

/// Assert that setting the domain name through `bin` is denied for a non-root user
#[cfg(unix)]
fn assert_set_requires_root(bin: &str) {
    if is_root() {
        return;
    }
    Command::cargo_bin(bin)
        .unwrap()
        .arg("uutils-test-domain")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "you must be root to change the domain name",
        ));
}

#[test]
#[cfg(unix)]
fn domainname_prints_nis_domain() {
    assert_prints_nis_domain("domainname");
}

#[test]
#[cfg(unix)]
fn nisdomainname_prints_nis_domain() {
    assert_prints_nis_domain("nisdomainname");
}

#[test]
#[cfg(unix)]
fn ypdomainname_prints_nis_domain() {
    assert_prints_nis_domain("ypdomainname");
}

#[test]
#[cfg(unix)]
fn dnsdomainname_prints_dns_domain() {
    let expected = stdout_of("hostname", &["-d"]);
    Command::cargo_bin("dnsdomainname")
        .unwrap()
        .assert()
        .success()
        .stdout(expected);
}

#[test]
#[cfg(unix)]
fn domainname_set_requires_root() {
    assert_set_requires_root("domainname");
}

#[test]
#[cfg(unix)]
fn nisdomainname_set_requires_root() {
    assert_set_requires_root("nisdomainname");
}

#[test]
#[cfg(unix)]
fn ypdomainname_set_requires_root() {
    assert_set_requires_root("ypdomainname");
}

#[test]
#[cfg(unix)]
fn test_set_empty_domain_name() {
    Command::cargo_bin("domainname")
        .unwrap()
        .arg("  ")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "the specified domain name is invalid",
        ));
}
