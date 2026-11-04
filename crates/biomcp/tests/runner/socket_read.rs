//! A signal interrupts an attempt, not the caller's read or deadline measurement.
use std::io::{self, Read};

pub(crate) fn read(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    loop {
        match reader.read(buffer) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => return result,
        }
    }
}

tool_only! {
pub(crate) fn proof() -> Result<(), String> {
    struct Scripted(std::vec::IntoIter<io::Result<usize>>);
    impl Read for Scripted {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            self.0.next().unwrap_or_else(|| Err(io::Error::other("unexpected extra read")))
        }
    }
    let mut reader = Scripted(vec![Err(io::ErrorKind::Interrupted.into()), Ok(0)].into_iter());
    if read(&mut reader, &mut [0]).map_err(|e| e.to_string())? != 0 || reader.0.next().is_some() {
        return Err("interrupted read did not continue to EOF".into());
    }
    for kind in [
        io::ErrorKind::TimedOut,
        io::ErrorKind::WouldBlock,
        io::ErrorKind::ConnectionReset,
        io::ErrorKind::BrokenPipe,
    ] {
        let mut reader = Scripted(vec![Err(kind.into())].into_iter());
        if read(&mut reader, &mut [0]).err().map(|e| e.kind()) != Some(kind) {
            return Err(format!("read did not preserve {kind:?}"));
        }
    }
    Ok(())
}
}
