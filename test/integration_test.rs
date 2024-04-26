use assert_cmd::Command; // To run the command line application
use std::fs;
use std::io::Write;

// Path constants for the tests directory and test files
const TEST_DIR: &str = "tests/test_cases/";
const CNF_EXT: &str = ".cnf";
const GOLDEN_EXT: &str = ".golden";

fn run_test_case(test_name: &str) {
    let cnf_path = format!("{}{}{}", TEST_DIR, test_name, CNF_EXT);
    let golden_path = format!("{}{}{}", TEST_DIR, test_name, GOLDEN_EXT);
    let output_path = format!("{}{}.out", TEST_DIR, test_name);
    let log_path = format!("{}{}.log", TEST_DIR, test_name);
    let err_path = format!("{}{}.err", TEST_DIR, test_name);

    // Cleanup before running
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&log_path);
    let _ = fs::remove_file(&err_path);

    let mut cmd = Command::cargo_bin("babysub-rust").unwrap();
    cmd.arg("-s").arg(&cnf_path).arg(&output_path);
    cmd.assert().success();

    // Read output to log file and error to err file
    let output = cmd.output().unwrap();
    let mut log_file = fs::File::create(log_path).unwrap();
    let mut err_file = fs::File::create(err_path).unwrap();
    log_file.write_all(&output.stdout).unwrap();
    err_file.write_all(&output.stderr).unwrap();

    // Compare hash-signature with golden file
    let output_content = fs::read_to_string(output_path).unwrap();
    let golden_content = fs::read_to_string(golden_path).unwrap();

    let output_hash = output_content
        .lines()
        .find(|line| line.starts_with("c hash-signature"))
        .unwrap();
    let golden_hash = golden_content
        .lines()
        .find(|line| line.starts_with("c hash-signature"))
        .unwrap();

    assert_eq!(
        output_hash, golden_hash,
        "Hash signatures do not match for test: {}",
        test_name
    );
}

#[test]
fn test_empty() {
    run_test_case("empty");
}

#[test]
fn test_binbin1() {
    run_test_case("binbin1");
}

#[test]
fn test_binbin2() {
    run_test_case("binbin2");
}

#[test]
fn test_inconsistent1() {
    run_test_case("inconsistent1");
}

#[test]
fn test_inconsistent2() {
    run_test_case("inconsistent2");
}

#[test]
fn test_trivial1() {
    run_test_case("trivial1");
}

#[test]
fn test_trivial2() {
    run_test_case("trivial2");
}
