//! Exact native JSON page counts share the checked request, recording and cache path.
use crate::BadReply;
/// Provider-given inclusive row bounds and exact total, never inferred from page length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentRange {
    /// First and last returned row, or no bounds for an empty range.
    pub bounds: Option<(u64, u64)>,
    /// Exact source collection total.
    pub total: u64,
}
impl ContentRange {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let (bounds, total) = text.split_once('/')?;
        let total = decimal(total)?;
        let bounds = if bounds == "*" {
            None
        } else {
            let (first, last) = bounds.split_once('-')?;
            let (first, last) = (decimal(first)?, decimal(last)?);
            if first > last || last >= total {
                return None;
            }
            Some((first, last))
        };
        Some(Self { bounds, total })
    }
    /// Check the requested offset, actual row count and native success status together.
    pub fn check(
        self,
        offset: u32,
        limit: u32,
        returned: usize,
        status: u16,
    ) -> Result<(), BadReply> {
        let n = u64::try_from(returned).map_err(|_| BadReply)?;
        let coherent = if n == 0 {
            self.bounds.is_none() && u64::from(offset) >= self.total
        } else {
            self.bounds
                == Some((
                    u64::from(offset),
                    u64::from(offset)
                        .checked_add(n)
                        .and_then(|v| v.checked_sub(1))
                        .ok_or(BadReply)?,
                ))
        };
        if !coherent || n > u64::from(limit) || status != if n == self.total { 200 } else { 206 } {
            return Err(BadReply);
        }
        Ok(())
    }
}
fn decimal(text: &str) -> Option<u64> {
    (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}
