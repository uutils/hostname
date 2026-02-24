use uutests::new_ucmd;

pub const TESTS_BINARY: &str = env!("CARGO_BIN_EXE_hostname");

// Use the ctor attribute to run this function before any tests
#[ctor::ctor]
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
