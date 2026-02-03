use assert_cmd::Command as AssertCommand;

// Note: use `cargo::cargo_bin_cmd!` macro to obtain the built binary to run.
// We avoid `Command::cargo_bin` (deprecated) and prefer the macro which is
// compatible with custom Cargo build directories.
fn run_output_from_cmd(cmd: &mut AssertCommand) -> (i32, String, String) {
    let output = cmd.output().unwrap();
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn normalize_output(output: &(i32, String, String)) -> (i32, String, String) {
    use regex::Regex;

    // Normalize CRLF to LF so comparisons succeed across platforms.
    let mut stdout = output.1.replace("\r\n", "\n");
    let mut stderr = output.2.replace("\r\n", "\n");

    // Replace absolute/relative paths to the built binaries (handles both
    // POSIX and Windows path separators and optional .exe suffix).
    let path_re = Regex::new(r"[A-Za-z]:[\\/][^\s]*target(?:[\\/](?:debug|release))[\\/][^\s]+|/.*/target/(?:debug|release)/[^\s]+").unwrap();
    stdout = path_re.replace_all(&stdout, "hostname").to_string();
    stderr = path_re.replace_all(&stderr, "hostname").to_string();

    // Replace any bare wrapper binary name occurrences (e.g., 'domainname' or 'domainname.exe') with 'hostname'
    let name_re =
        Regex::new(r"\b(?:dnsdomainname|domainname|ypdomainname|nisdomainname)\b(?:\.exe)?")
            .unwrap();
    stdout = name_re.replace_all(&stdout, "hostname").to_string();
    stderr = name_re.replace_all(&stderr, "hostname").to_string();

    (output.0, stdout, stderr)
}

#[test]
fn dnsdomainname_matches_hostname_d() {
    let a = normalize_output(&run_output_from_cmd(
        &mut assert_cmd::cargo::cargo_bin_cmd!("dnsdomainname").arg("--help"),
    ));
    let b = normalize_output(&run_output_from_cmd(
        &mut assert_cmd::cargo::cargo_bin_cmd!("hostname")
            .arg("-d")
            .arg("--help"),
    ));
    assert_eq!(a, b);
}

#[test]
fn domainname_matches_hostname_y() {
    let a = normalize_output(&run_output_from_cmd(
        &mut assert_cmd::cargo::cargo_bin_cmd!("domainname").arg("--help"),
    ));
    let b = normalize_output(&run_output_from_cmd(
        &mut assert_cmd::cargo::cargo_bin_cmd!("hostname")
            .arg("-y")
            .arg("--help"),
    ));
    assert_eq!(a, b);
}

#[test]
fn nisdomainname_matches_hostname_y() {
    let a = normalize_output(&run_output_from_cmd(
        &mut assert_cmd::cargo::cargo_bin_cmd!("nisdomainname").arg("--help"),
    ));
    let b = normalize_output(&run_output_from_cmd(
        &mut assert_cmd::cargo::cargo_bin_cmd!("hostname")
            .arg("-y")
            .arg("--help"),
    ));
    assert_eq!(a, b);
}

#[test]
fn ypdomainname_matches_hostname_y() {
    let a = normalize_output(&run_output_from_cmd(
        &mut assert_cmd::cargo::cargo_bin_cmd!("ypdomainname").arg("--help"),
    ));
    let b = normalize_output(&run_output_from_cmd(
        &mut assert_cmd::cargo::cargo_bin_cmd!("hostname")
            .arg("-y")
            .arg("--help"),
    ));
    assert_eq!(a, b);
}

#[test]
fn ping_wrapper_tests() {
    // sanity test to ensure this test file is discovered by the harness
    assert!(true);
}
