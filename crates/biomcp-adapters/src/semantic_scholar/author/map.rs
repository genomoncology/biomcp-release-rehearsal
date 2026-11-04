//! Retain native pagination privately and expose mapped author candidates.
use biodata::AuthorHeader;
use serde::{Deserialize, Serialize};
/// Stored source page with a private native representation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Page(super::raw::Page);
impl Page {
    pub(super) fn new(page: super::raw::Page) -> Self {
        Self(page)
    }
    /// Native start offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Native next offset, only when supplied.
    pub fn next(&self) -> Option<u32> {
        self.0.next
    }
    /// Native matching records, when supplied.
    pub fn total(&self) -> Option<u64> {
        self.0.total
    }
    /// Accepted source identities in this page.
    pub fn len(&self) -> usize {
        self.0.data.len()
    }
    /// Whether this page has no accepted source identities.
    pub fn is_empty(&self) -> bool {
        self.0.data.is_empty()
    }
    /// Actual refused source identities.
    pub fn refusals(&self) -> crate::Refusal {
        self.0.refusals
    }
    /// Restore refusal provenance stored beside a page.
    pub fn restore_refusals(&mut self, refused: bool) {
        self.0.refusals = if refused {
            crate::Refusal::Refused
        } else {
            crate::Refusal::None
        };
    }
    /// Map accepted metadata with field-local source problems.
    pub fn headers(self) -> crate::Mapped<Box<[AuthorHeader]>> {
        let mut problems = self.0.problems;
        let value = self
            .0
            .data
            .into_iter()
            .map(|row| row.header(&mut problems))
            .collect();
        crate::Mapped {
            value,
            problems,
            refusals: self.0.refusals,
        }
    }
}
