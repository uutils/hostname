// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(unix)]
use crate::is_root;
use uutests::new_ucmd;
use uutests::util::{AtPath, TestScenario, UCommand};
use uutests::util_name;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_help_flag() {
    new_ucmd!()
        .arg("--help")
        .succeeds()
        .stdout_contains("Usage: hostname");
}

#[test]
fn test_version_flag() {
    new_ucmd!()
        .arg("--version")
        .succeeds()
        // uucore derives the util name from the binary's file name, which
        // includes the `.exe` suffix on Windows.
        .stdout_contains(format!("hostname{} ", std::env::consts::EXE_SUFFIX));
}

/// Like `at_and_ucmd!()`, but without the util name `new_ucmd!()` prepends,
/// which the `hostname` binary would take as the host name to set.
fn at_and_hostname_cmd() -> (AtPath, UCommand) {
    let ts = TestScenario::new(util_name!());
    let cmd = ts.cmd(&ts.bin_path);
    (ts.fixtures.clone(), cmd)
}

#[test]
fn test_nis_conflicts_with_get_flags() {
    for flag in ["-s", "-d"] {
        at_and_hostname_cmd()
            .1
            .args(&["-y", flag])
            .fails()
            .code_is(1)
            .stderr_contains("the argument '--yp' cannot be used with");
    }
}

#[test]
#[cfg(unix)]
fn test_set_nis_domain_name_requires_root() {
    if is_root() {
        return;
    }
    at_and_hostname_cmd()
        .1
        .args(&["-y", "uutils-test-domain"])
        .fails()
        .code_is(1)
        .stderr_contains("you must be root to change the domain name");
}

#[test]
#[cfg(unix)]
fn test_set_host_name_requires_root() {
    if is_root() {
        return;
    }
    at_and_hostname_cmd()
        .1
        .arg("uutils-test-host")
        .fails()
        .code_is(1)
        .stderr_contains("you must be root to change the host name");
}

#[test]
#[cfg(unix)]
fn test_set_blank_host_name() {
    at_and_hostname_cmd()
        .1
        .arg("  ")
        .fails()
        .code_is(1)
        .stderr_contains("the specified hostname is invalid");
}

#[test]
#[cfg(unix)]
fn test_set_host_name_from_blank_file() {
    let (at, mut ucmd) = at_and_hostname_cmd();
    at.write("name", "  \n");
    ucmd.arg("-F")
        .arg("name")
        .fails()
        .code_is(1)
        .stderr_contains("the specified hostname is invalid");
}

#[test]
#[cfg(unix)]
fn test_set_domain_name_from_blank_file() {
    let (at, mut ucmd) = at_and_hostname_cmd();
    at.write("name", "  \n");
    ucmd.arg("-y")
        .arg("-F")
        .arg("name")
        .fails()
        .code_is(1)
        .stderr_contains("the specified domain name is invalid");
}

#[test]
#[cfg(unix)]
fn test_set_host_name_from_padded_file() {
    if is_root() {
        return;
    }
    // Reaching the permission check means the padding was trimmed and the
    // name passed validation.
    let (at, mut ucmd) = at_and_hostname_cmd();
    at.write("name", "  uutils-test-host  \n");
    ucmd.arg("-F")
        .arg("name")
        .fails()
        .code_is(1)
        .stderr_contains("you must be root to change the host name");
}
