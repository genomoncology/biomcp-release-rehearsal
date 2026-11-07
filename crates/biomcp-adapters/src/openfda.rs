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
    fn data_date(&self) -> Option<String> {
        match self {
            Self::Page { meta, .. } => meta.last_updated.map(|date| date.to_string()),
            Self::Absent { .. } => None,
        }
    }
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

/// Recover the native total only when a nonzero page is reported absent.
/// The probe keeps the selector and sort of the requested page and omits `skip`,
/// as every adapter's first page may.
pub(crate) fn page<T: Item>(
    source: biomcp_types::Source,
    leg: &biomcp_fetch::Leg<'_>,
    endpoint: &biomcp_fetch::Endpoint,
    args: &[(&str, biomcp_fetch::Arg<'_>)],
    offset: u32,
    limit: u32,
    check: &dyn Fn(&Envelope<T>) -> Result<(), BadReply>,
) -> Result<biomcp_fetch::Received<Envelope<T>>, biomcp_fetch::RequestError> {
    use biomcp_fetch::{Arg, Input, Read};
    let mut received = leg.call(endpoint, Input::Params(args), Read::json().check(check))?;
    if offset == 0 || !matches!(received.value, Envelope::Absent { .. }) {
        return Ok(received);
    }
    let mut probe = Vec::new();
    for (name, value) in args {
        if *name == "skip" {
            continue;
        }
        let value = match (*name, value) {
            ("limit", _) => Arg::Num(1),
            (_, Arg::Text(value)) => Arg::Text(value),
            (_, Arg::Num(value)) => Arg::Num(*value),
            (_, Arg::Ids(value)) => Arg::Ids(value),
            (_, Arg::Contact(value)) => Arg::Contact(value),
        };
        probe.push((*name, value));
    }
    let first = leg.call(
        endpoint,
        Input::Params(&probe),
        Read::<Envelope<T>>::json().check(&|reply| match reply {
            Envelope::Page { meta, .. } if meta.results.skip != 0 || meta.results.limit != 1 => {
                Err(BadReply)
            }
            _ => Ok(()),
        }),
    )?;
    if let Envelope::Page { mut meta, .. } = first.value {
        if u64::from(offset) < meta.results.total {
            return Err(biomcp_fetch::bad_reply(source, Some(404)));
        }
        meta.results.skip = offset;
        meta.results.limit = limit;
        received.value = Envelope::Page {
            meta,
            results: Vec::new(),
        };
    }
    Ok(received)
}
