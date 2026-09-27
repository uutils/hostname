use uutests::new_ucmd;

pub const TESTS_BINARY: &str = env!("CARGO_BIN_EXE_hostname");

// Use the ctor attribute to run this function before any tests
#[ctor::ctor(unsafe)]
fn init() {
    unsafe {
        // Necessary for uutests to be able to find the binary
        std::env::set_var("UUTESTS_BINARY_PATH", TESTS_BINARY);
    }
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_help_flag() {
    new_ucmd!().arg("--help").succeeds();
}

#[test]
fn test_version_flag() {
    new_ucmd!().arg("--version").succeeds();
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

#[test]
#[cfg(unix)]
fn test_set_domain_name_requires_root() {
    if is_root() {
        return;
    }
    assert_cmd::Command::cargo_bin("domainname")
        .unwrap()
        .arg("uutils-test-domain")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "you must be root to change the domain name",
        ));
}

#[test]
#[cfg(unix)]
fn test_set_empty_domain_name() {
    assert_cmd::Command::cargo_bin("domainname")
        .unwrap()
        .arg("  ")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("is invalid"));
}
