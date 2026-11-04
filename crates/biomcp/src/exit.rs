//! Consumed command outcomes and their process exit codes.
use std::{io, process::ExitCode};

pub(crate) enum Status {
    Success,
    Usage,
    Expired,
    Absent,
    Failed,
    Local,
}

pub(crate) fn finish(result: io::Result<()>, status: Status) -> ExitCode {
    match result {
        Ok(()) => ExitCode::from(status.code()),
        Err(_) => {
            crate::output::diagnostic(b"biomcp: output failed\n");
            ExitCode::from(Status::Local.code())
        }
    }
}

pub(crate) fn failure(error: io::Error) -> ExitCode {
    crate::output::diagnostic(format!("biomcp: {error}\n").as_bytes());
    ExitCode::from(Status::Local.code())
}

impl From<biomcp_types::Status> for Status {
    fn from(status: biomcp_types::Status) -> Self {
        match status {
            biomcp_types::Status::Complete | biomcp_types::Status::Partial => Self::Success,
            biomcp_types::Status::Absent => Self::Absent,
            biomcp_types::Status::Failed => Self::Failed,
        }
    }
}

impl Status {
    pub(crate) fn code(&self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Absent => 1,
            Self::Usage => 2,
            Self::Expired => 3,
            Self::Failed => 4,
            Self::Local => 5,
        }
    }
}
pub(crate) fn reference() -> String {
    let mut text = String::from("Process exit codes:\n");
    for (status, description) in [
        (Status::Success, "complete or partial reply"),
        (Status::Absent, "no matching record"),
        (Status::Usage, "invalid command or arguments"),
        (Status::Expired, "expired result pool"),
        (Status::Failed, "source request failed"),
        (
            Status::Local,
            "local configuration, storage or output failed",
        ),
    ] {
        text.push_str(&format!("{}: {description}\n", status.code()));
    }
    if let Some(schema) = crate::schema::find("biomcp.error/4")
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
    {
        let codes = schema
            .pointer("/$defs/Reply_ErrorCode/enum")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        text.push_str(&format!(
            "Error codes: {codes}. Source problems and reasons appear in their reply fields."
        ));
    }
    text
}
