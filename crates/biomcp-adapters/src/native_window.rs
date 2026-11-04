//! Accepted rows and the native positions traversed before identity refusals.
/// Internal adapter result; native counts never define public retrieved counts.
#[derive(Debug)]
pub struct NativeWindow<T> {
    /// Accepted records in source order.
    pub rows: Vec<T>,
    offset: u32,
    limit: u32,
    total: Option<u64>,
    received: u32,
    refusals: crate::Refusal,
}
impl<T> NativeWindow<T> {
    /// Bind accepted records to counts measured before identity filtering.
    pub fn new(
        rows: Vec<T>,
        offset: u32,
        limit: u32,
        total: Option<u64>,
        received: u32,
        refusals: crate::Refusal,
    ) -> Self {
        Self {
            rows,
            offset,
            limit,
            total,
            received,
            refusals,
        }
    }
    /// Select native slots before removing refused records.
    pub fn local(rows: Vec<Option<T>>, offset: u32, limit: u32) -> Self {
        let total = rows.len() as u64;
        let slots: Vec<_> = rows
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        let received = slots.len() as u32;
        let mut rows = Vec::new();
        let mut refusals = crate::Refusal::None;
        for slot in slots {
            match slot {
                Some(row) => rows.push(row),
                None => refusals = crate::Refusal::Refused,
            }
        }
        Self::new(rows, offset, limit, Some(total), received, refusals)
    }
    /// Actual record refusals in this selected window.
    pub fn refusals(&self) -> crate::Refusal {
        self.refusals
    }
    /// Retain actual record refusals from a later mapping stage.
    pub fn include_refusals(&mut self, refusals: crate::Refusal) {
        self.refusals.include(refusals);
    }
    /// Native selector offset.
    pub fn offset(&self) -> u32 {
        self.offset
    }
    /// Requested native window size.
    pub fn limit(&self) -> u32 {
        self.limit
    }
    /// Native selector total, when the provider supplies one.
    pub fn total(&self) -> Option<u64> {
        self.total
    }
    /// Native slots examined, including refused identities.
    pub fn received(&self) -> u32 {
        self.received
    }
    /// Whether the native window contained no rows before filtering.
    pub fn provider_empty(&self) -> bool {
        self.received == 0
    }
    /// Admit valid native empty pages or pages with at least one accepted identity.
    pub fn identities_admitted(&self) -> bool {
        self.refusals
            .identities_admitted(self.rows.len(), self.received as usize)
    }
    /// Whether native rows indicate a further window, independent of accepted rows.
    pub fn has_more(&self) -> bool {
        !self.provider_empty()
            && self.total.map_or(self.received == self.limit, |total| {
                u64::from(self.offset) + u64::from(self.received) < total
            })
    }
    /// Advance by native slots without overflowing the selector offset.
    pub fn next_offset(&self) -> Option<u32> {
        self.has_more()
            .then(|| self.offset.checked_add(self.received))
            .flatten()
    }
    /// Keep native traversal facts while mapping accepted records for a caller.
    pub fn with_rows<U>(self, rows: Vec<U>) -> NativeWindow<U> {
        NativeWindow::new(
            rows,
            self.offset,
            self.limit,
            self.total,
            self.received,
            self.refusals,
        )
    }
}

impl crate::Refusal {
    /// Admit an accepted identity or a native collection with no refused records.
    pub(crate) fn identities_admitted(self, accepted: usize, received: usize) -> bool {
        accepted > 0 || (received == 0 && self == Self::None)
    }
}
