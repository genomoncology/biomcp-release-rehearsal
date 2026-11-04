//! The command output boundary.
use std::io::{self, Write};

pub(crate) fn command(outcome: &crate::command::Outcome) -> io::Result<()> {
    bytes(outcome.bytes()?)
}

pub(crate) fn bytes(bytes: &[u8]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    match stdout.write_all(bytes).and_then(|()| stdout.flush()) {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}

pub(crate) fn diagnostic(message: &[u8]) {
    let _ = io::stderr().lock().write_all(message);
}
