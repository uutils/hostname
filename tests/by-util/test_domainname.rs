// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;
use uutests::util::TestScenario;
use uutests::util_name;

// Get the domainname binary path
const DOMAINNAME_BINARY: &str = env!("CARGO_BIN_EXE_domainname");

// Helper function to create a command for the domainname binary
fn domainname_cmd() -> uutests::util::UCommand {
    TestScenario::new(util_name!()).cmd(DOMAINNAME_BINARY)
}

fn domainname_at_and_cmd() -> (uutests::util::AtPath, uutests::util::UCommand) {
    let ts = ::uutests::util::TestScenario::new(util_name!());
    (ts.fixtures.clone(), ts.cmd(DOMAINNAME_BINARY))
}

#[test]
fn test_invalid_arg() {
    domainname_cmd()
        .arg("--this-is-an-invalid-argument")
        .fails()
        .code_is(1);
}

#[test]
fn test_help() {
    domainname_cmd()
        .arg("--help")
        .succeeds()
        .no_stderr()
        .stdout_contains("show or set the system's NIS/YP domain name")
        .stdout_contains("Usage:");
}

#[test]
fn test_get_domainname() {
    domainname_cmd().succeeds().no_stderr();
}

#[test]
fn test_set_domainname_without_superuser() {
    domainname_cmd()
        .arg("random-domainname")
        .fails()
        .stderr_contains("you must be root to change the domain name");
}

#[test]
fn test_set_domainname_from_invalid_file() {
    domainname_cmd()
        .arg("-F invalid-path")
        .fails()
        .stderr_contains("No such file or directory");
}

#[test]
fn test_set_invalid_domainname() {
    domainname_cmd()
        .arg("invalid_domain_name")
        .fails()
        .stderr_contains("the specified domainname is invalid");
}

#[test]
fn test_set_domainname_from_empty_file() {
    let (at, mut cmd) = domainname_at_and_cmd();
    let file = "filename";
    at.touch(file);
    cmd.arg("-F")
        .arg(file)
        .fails()
        .stderr_contains("the specified domainname is invalid");
}

#[test]
fn test_set_domainname_from_file_without_superuser() {
    let (at, mut cmd) = domainname_at_and_cmd();
    let file = "filename";
    at.write(file, "example.com");
    cmd.arg("-F")
        .arg(file)
        .fails()
        .stderr_contains("you must be root to change the domain name");
}
