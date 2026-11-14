// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uutests::new_ucmd;

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

/// Setting a name as root would actually rename the machine running the tests.
#[cfg(unix)]
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[test]
#[cfg(unix)]
fn test_set_host_name_requires_root() {
    if is_root() {
        return;
    }
    assert_cmd::Command::cargo_bin("hostname")
        .unwrap()
        .arg("uutils-test-host")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "you must be root to change the host name",
        ));
}

/// Write `contents` to a temporary file for use with `-F`.
#[cfg(unix)]
fn name_file(contents: &str) -> tempfile::NamedTempFile {
    use std::io::Write;

    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(contents.as_bytes()).unwrap();
    file
}

#[test]
#[cfg(unix)]
fn test_set_blank_host_name() {
    assert_cmd::Command::cargo_bin("hostname")
        .unwrap()
        .arg("  ")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "the specified hostname is invalid",
        ));
}

#[test]
#[cfg(unix)]
fn test_set_host_name_from_blank_file() {
    let file = name_file("  \n");
    assert_cmd::Command::cargo_bin("hostname")
        .unwrap()
        .arg("-F")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "the specified hostname is invalid",
        ));
}

#[test]
#[cfg(unix)]
fn test_set_domain_name_from_blank_file() {
    let file = name_file("  \n");
    assert_cmd::Command::cargo_bin("hostname")
        .unwrap()
        .arg("-y")
        .arg("-F")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "the specified domain name is invalid",
        ));
}

#[test]
#[cfg(unix)]
fn test_set_host_name_from_padded_file() {
    if is_root() {
        return;
    }
    // Reaching the permission check means the padding was trimmed and the
    // name passed validation.
    let file = name_file("  uutils-test-host  \n");
    assert_cmd::Command::cargo_bin("hostname")
        .unwrap()
        .arg("-F")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "you must be root to change the host name",
        ));
}
