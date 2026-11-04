//! Copy source recordings from the common public fixture root.
use super::recordings;
/// Copy a recording tree while mutating each JSON exchange.
pub fn copy_capture(
    capture: &str,
    target: &std::path::Path,
    change: impl FnMut(&str, &mut serde_json::Value),
) -> Result<(), Box<dyn std::error::Error>> {
    recordings::copy_capture(
        &super::fixtures::fixtures().join("replies").join(capture),
        target,
        change,
    )
}
