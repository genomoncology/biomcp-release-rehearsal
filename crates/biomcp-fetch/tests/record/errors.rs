use super::*;

#[test]
fn recording_io_conversion_retains_kind_without_error_text() {
    let error = RecordError::from(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "synthetic-private-path",
    ));
    assert_eq!(error, RecordError::Io(std::io::ErrorKind::PermissionDenied));
    assert_eq!(error.to_string(), "fetch.record_io");
    assert!(!format!("{error:?}").contains("synthetic-private-path"));
}
