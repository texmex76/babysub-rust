use assert_cmd::Command; // To run the command line application
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Output;

// Path constants for the tests directory and test files
const TEST_DIR: &str = "test/test_cases/";
const CNF_EXT: &str = ".cnf";
const GOLDEN_EXT: &str = ".golden";

// Function to find the path to the executable
fn find_executable() -> Result<String, Box<dyn std::error::Error>> {
    let target_dir = Path::new("target/debug/");
    let entries = fs::read_dir(target_dir)?;
    for entry in entries {
        let path = entry?.path();
        if path.is_file() && path.file_name().unwrap_or_default() == "babysub-rust" {
            return Ok(path.to_string_lossy().to_string());
        }
    }
    Err("Executable 'babysub-rust' not found in target/debug/".into())
}

fn run_test_case(test_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cnf_path = Path::new(TEST_DIR).join(test_name).with_extension(CNF_EXT);
    let golden_path = Path::new(TEST_DIR)
        .join(test_name)
        .with_extension(GOLDEN_EXT);
    let output_path = Path::new(TEST_DIR).join(test_name).with_extension("out");
    let log_path = Path::new(TEST_DIR).join(test_name).with_extension("log");
    let err_path = Path::new(TEST_DIR).join(test_name).with_extension("err");

    // Cleanup before running
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&log_path);
    let _ = fs::remove_file(&err_path);

    let executable_path = find_executable()?;
    let mut cmd = Command::new(executable_path);
    cmd.arg("-s").arg(&cnf_path).arg(&output_path);

    let output: Output = cmd.output()?;

    // Write output to log file and error to err file
    fs::File::create(&log_path)?.write_all(&output.stdout)?;
    fs::File::create(&err_path)?.write_all(&output.stderr)?;

    // Compare hash-signature with golden file
    let output_content = fs::read_to_string(&output_path)?;
    let golden_content = fs::read_to_string(&golden_path)?;

    let output_hash = output_content
        .lines()
        .find(|line| line.starts_with("c hash-signature"))
        .ok_or("Output hash-signature not found")?;
    let golden_hash = golden_content
        .lines()
        .find(|line| line.starts_with("c hash-signature"))
        .ok_or("Golden hash-signature not found")?;

    if output_hash != golden_hash {
        return Err(format!("Hash signatures do not match for test: {}", test_name).into());
    }

    Ok(())
}

#[test]
fn test_empty() -> Result<(), Box<dyn std::error::Error>> {
    run_test_case("empty")
}

#[test]
fn test_binbin1() -> Result<(), Box<dyn std::error::Error>> {
    run_test_case("binbin1")
}

#[test]
fn test_binbin2() -> Result<(), Box<dyn std::error::Error>> {
    run_test_case("binbin2")
}

#[test]
fn test_inconsistent1() -> Result<(), Box<dyn std::error::Error>> {
    run_test_case("inconsistent1")
}

#[test]
fn test_inconsistent2() -> Result<(), Box<dyn std::error::Error>> {
    run_test_case("inconsistent2")
}

#[test]
fn test_trivial1() -> Result<(), Box<dyn std::error::Error>> {
    run_test_case("trivial1")
}

#[test]
fn test_trivial2() -> Result<(), Box<dyn std::error::Error>> {
    run_test_case("trivial2")
}
