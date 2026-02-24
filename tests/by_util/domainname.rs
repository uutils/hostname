// Tests for the domainname utilities: dnsdomainname, domainname, nisdomainname, and ypdomainname wrapper binaries,
// These binaries are built from src/uu/*/src/main.rs and delegates to hostname.

use assert_cmd::Command;
use predicates::prelude::*;

/// Assert that `--help` succeeds and prints either the binary name or Usage text
fn assert_help(bin: &str) {
    Command::cargo_bin(bin)
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(bin)
                .or(predicate::str::contains("Usage")),
        );
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