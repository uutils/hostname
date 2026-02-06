// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;
use uutests::util::TestScenario;

// Get the dnsdomainname binary path
const DNSDOMAINNAME_BINARY: &str = env!("CARGO_BIN_EXE_dnsdomainname");

// Helper function to create a command for the dnsdomainname binary
fn dnsdomainname_cmd() -> uutests::util::UCommand {
    TestScenario::new("test").cmd(DNSDOMAINNAME_BINARY)
}

#[test]
fn test_invalid_arg() {
    dnsdomainname_cmd()
        .arg("--definitely-invalid")
        .fails()
        .code_is(1);
}

#[test]
fn test_help() {
    dnsdomainname_cmd()
        .arg("--help")
        .succeeds()
        .no_stderr()
        .stdout_contains("Display the DNS domain name")
        .stdout_contains("Usage:");
}

#[test]
fn test_version() {
    dnsdomainname_cmd()
        .arg("--version")
        .succeeds()
        .no_stderr()
        .stdout_contains("0.0.1");
}

#[test]
fn test_no_arguments_required() {
    // dnsdomainname should not accept any positional arguments
    dnsdomainname_cmd()
        .arg("some_argument")
        .fails()
        .code_is(1)
        .stderr_contains("unexpected argument");
}
