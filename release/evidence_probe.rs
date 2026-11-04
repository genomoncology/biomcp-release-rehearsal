//! Exercise the retained-evidence owner without a build checkout in the install container.
#[path = "../crates/biomcp-test-support/src/evidence.rs"]
mod evidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let expected = std::path::PathBuf::from(std::env::args().nth(1).expect("expected evidence path"));
    let directory = evidence::directory().map_err(|error| {
        eprintln!("compiled checkout fallback /source/crates/biomcp-test-support/../../target/fixture-evidence: {error}");
        error
    })?;
    assert_eq!(directory, expected);
    std::fs::write(directory.join("retained-marker"), b"retained evidence")?;
    let scratch = std::env::var_os("TMPDIR").expect("isolated temporary directory");
    std::fs::remove_dir_all(scratch)?;
    assert_eq!(std::fs::read(directory.join("retained-marker"))?, b"retained evidence");
    println!("retained evidence survives scratch cleanup: {}", directory.display());
    Ok(())
}
