//! Measure the bytes the command and MCP actually emit.
use super::{Write, ceiling};

pub(in crate::runner::cell_line) fn measure_bytes(
    phase: &str,
    surface: &str,
    bytes: usize,
    measure: bool,
) -> Result<(), String> {
    let _ = writeln!(
        std::io::stderr(),
        "CELL_LINE_MEASURE {phase} [{surface}] {bytes}"
    );
    if !measure && bytes > ceiling(phase, surface)? {
        return Err(format!("{phase} [{surface}] exceeds measured cap"));
    }
    Ok(())
}
