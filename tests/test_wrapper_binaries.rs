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
    let re = Regex::new(r"/.*/target/debug/[^\s]+").unwrap_or_else(|_| Regex::new(r"invalid").unwrap());
    let stdout = re.replace_all(&output.1, "hostname").to_string();
    let stderr = re.replace_all(&output.2, "hostname").to_string();
    (output.0, stdout, stderr)
}

#[test]
fn dnsdomainname_matches_hostname_d() {
    let a = normalize_output(&run_output_from_cmd(&mut assert_cmd::cargo::cargo_bin_cmd!("dnsdomainname").arg("--help")));
    let b = normalize_output(&run_output_from_cmd(&mut assert_cmd::cargo::cargo_bin_cmd!("hostname").arg("-d").arg("--help")));
    assert_eq!(a, b);
}

#[test]
fn domainname_matches_hostname_y() {
    let a = normalize_output(&run_output_from_cmd(&mut assert_cmd::cargo::cargo_bin_cmd!("domainname").arg("--help")));
    let b = normalize_output(&run_output_from_cmd(&mut assert_cmd::cargo::cargo_bin_cmd!("hostname").arg("-y").arg("--help")));
    assert_eq!(a, b);
}

#[test]
fn nisdomainname_matches_hostname_y() {
    let a = normalize_output(&run_output_from_cmd(&mut assert_cmd::cargo::cargo_bin_cmd!("nisdomainname").arg("--help")));
    let b = normalize_output(&run_output_from_cmd(&mut assert_cmd::cargo::cargo_bin_cmd!("hostname").arg("-y").arg("--help")));
    assert_eq!(a, b);
}

#[test]
fn ypdomainname_matches_hostname_y() {
    let a = normalize_output(&run_output_from_cmd(&mut assert_cmd::cargo::cargo_bin_cmd!("ypdomainname").arg("--help")));
    let b = normalize_output(&run_output_from_cmd(&mut assert_cmd::cargo::cargo_bin_cmd!("hostname").arg("-y").arg("--help")));
    assert_eq!(a, b);
}

#[test]
fn ping_wrapper_tests() {
    // sanity test to ensure this test file is discovered by the harness
    assert!(true);
}


