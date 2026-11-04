use crate::{AssertionSource, QueryInvalid, RcvId, ScvId, Text, VcvId};
use serde::Serialize;
use std::fmt;
/// Native assertion selector, preserving its source grammar.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum AssertionId {
    /// ClinVar variation archive.
    Vcv(VcvId),
    /// ClinVar condition record.
    Rcv(RcvId),
    /// ClinVar submitted record.
    Scv(ScvId),
    /// Evidence repository document identity.
    Erepo(Text),
}
impl AssertionId {
    pub(super) fn parse(id: &Text, source: AssertionSource) -> Result<Self, QueryInvalid> {
        let value = id.as_str();
        let invalid = || QueryInvalid::new(&["assertion", "source"]);
        match source {
            AssertionSource::Clinvar if !value.contains('.') => value
                .parse()
                .map(Self::Vcv)
                .or_else(|_| value.parse().map(Self::Rcv))
                .or_else(|_| value.parse().map(Self::Scv))
                .map_err(|_| invalid()),
            AssertionSource::Erepo
                if value.strip_prefix("PCER").is_some_and(|n| {
                    n.len() == 9
                        && n.bytes().all(|b| b.is_ascii_digit())
                        && n.parse::<u64>().is_ok_and(|n| n > 0)
                }) =>
            {
                Ok(Self::Erepo(id.clone()))
            }
            _ => Err(invalid()),
        }
    }
}
impl fmt::Display for AssertionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vcv(id) => id.fmt(f),
            Self::Rcv(id) => id.fmt(f),
            Self::Scv(id) => id.fmt(f),
            Self::Erepo(id) => id.fmt(f),
        }
    }
}
