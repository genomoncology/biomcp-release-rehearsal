use super::{TrialSearch, TrialSource};

impl TrialSearch {
    /// Whether the source must search the registered condition phrase without expansion.
    pub fn exact_condition(&self) -> bool {
        self.0.exact_condition
    }

    /// Offer an explicit source-field restriction with all other filters retained.
    pub fn exact_condition_search(&self) -> Option<Self> {
        if self.source() != TrialSource::Ctgov
            || self.condition().is_empty()
            || self.exact_condition()
        {
            return None;
        }
        let mut next = self.clone();
        next.0.exact_condition = true;
        next.0.next_page = None;
        Some(next)
    }
}
