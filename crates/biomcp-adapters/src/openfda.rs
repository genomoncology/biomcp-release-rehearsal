//! Checked openFDA page envelopes with source-specific record admission.
use biodata::PartialDate;
use biomcp_fetch::{BadReply, Reply};
use serde::{Deserialize, de::DeserializeOwned};
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Envelope<T> {
    Page { meta: Meta, results: Vec<T> },
    Absent { error: Absence },
}
#[derive(Debug, Deserialize)]
pub(crate) struct Meta {
    pub(crate) last_updated: Option<PartialDate>,
    pub(crate) results: Counts,
}
#[derive(Debug, Deserialize)]
pub(crate) struct Counts {
    pub(crate) skip: u32,
    pub(crate) limit: u32,
    pub(crate) total: u64,
}
#[derive(Debug, Deserialize)]
pub(crate) struct Absence {
    code: String,
    message: String,
    #[serde(flatten)]
    extra: std::collections::BTreeMap<String, serde::de::IgnoredAny>,
}
impl Absence {
    pub(crate) fn valid(&self) -> bool {
        self.extra.is_empty() && self.code == "NOT_FOUND" && self.message == "No matches found!"
    }
}
pub(crate) trait Item: DeserializeOwned {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
    fn counts(meta: &Meta, len: usize) -> bool {
        meta.results.limit > 0
            && len as u64
                == meta
                    .results
                    .total
                    .saturating_sub(meta.results.skip.into())
                    .min(meta.results.limit.into())
    }
}
impl<T: Item> Reply for Envelope<T> {
    fn check(&self) -> Result<(), BadReply> {
        match self {
            Self::Page { meta, results } if T::counts(meta, results.len()) => {
                results.iter().try_for_each(Item::check)
            }
            Self::Absent { error } if error.valid() => Ok(()),
            _ => Err(BadReply),
        }
    }
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        if matches!(
            (self, status),
            (Self::Page { .. }, 200) | (Self::Absent { .. }, 404)
        ) {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}
