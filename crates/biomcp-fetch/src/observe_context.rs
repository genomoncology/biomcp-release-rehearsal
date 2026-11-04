//! Optional causal context for bounded source diagnostics, never request behavior.
use crate::{Reply, prepared::Prepared};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_IDENTITIES: usize = 10_000;

/// A source-owned identifier namespace and complete ordered selection.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Identities {
    /// Checked identifier system, such as PMID.
    pub namespace: String,
    /// Whole received or requested sequence; never silently sampled.
    pub ids: Vec<String>,
}
impl Identities {
    /// Bound diagnostic metadata; absent context cannot establish causation.
    pub fn bounded(namespace: &str, ids: Vec<String>) -> Option<Self> {
        (ids.len() <= MAX_IDENTITIES
            && ids
                .iter()
                .all(|id| !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control)))
        .then(|| Self {
            namespace: namespace.into(),
            ids,
        })
    }
}
/// Explicit optional evidence for discovery-driven request changes.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Provenance {
    /// SHA-256 of the received body, without retaining provider values.
    pub response_digest: Option<String>,
    /// Complete checked discovery result identities.
    pub discovery: Option<Identities>,
    /// Complete provider-owned sequences across distinct namespaces; no sampled IDs.
    #[serde(default)]
    pub discoveries: Option<Vec<Identities>>,
    /// Explicit request ID inputs, excluding other parameters and credentials.
    pub inputs: Option<Identities>,
    /// Equality witness for the credential-free request except its declared IDs.
    /// This cannot establish discovery causality without explicit identifiers.
    pub input_context: Option<String>,
}
impl Provenance {
    pub(crate) fn received<T: Reply>(
        prepared: &Prepared,
        body: &[u8],
        status: Option<u16>,
    ) -> Self {
        let (mut discovery, mut discoveries) = status
            .filter(|status| (200..300).contains(status))
            .and_then(|status| {
                let reply: T = serde_json::from_slice(body).ok()?;
                reply.check().ok()?;
                reply.check_status(status).ok()?;
                Some((reply.discovery(), reply.discoveries()))
            })
            .unwrap_or_default();
        let sequences = discoveries.as_deref().unwrap_or_default();
        let mut namespaces = std::collections::BTreeSet::new();
        let mut total = 0usize;
        let valid = discovery.iter().chain(sequences).all(|ids| {
            total = total.saturating_add(ids.ids.len());
            !ids.namespace.is_empty()
                && ids.namespace.len() <= 128
                && namespaces.insert(&ids.namespace)
                && Identities::bounded(&ids.namespace, ids.ids.clone()).is_some()
                && ids
                    .ids
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == ids.ids.len()
        });
        if !valid || total > MAX_IDENTITIES {
            discovery = None;
            discoveries = None;
        }
        let inputs = T::input_parameter().and_then(|(namespace, parameter)| {
            let url = url::Url::parse(&prepared.url).ok()?;
            let mut values = url
                .query_pairs()
                .filter(|(name, _)| name == parameter)
                .map(|(_, value)| value.into_owned())
                .collect::<Vec<_>>();
            if prepared.post.as_ref().is_some_and(|post| {
                post.request_content_type == "application/x-www-form-urlencoded"
            }) {
                values.extend(
                    url::form_urlencoded::parse(&prepared.body)
                        .filter(|(name, _)| name == parameter)
                        .map(|(_, value)| value.into_owned()),
                );
            }
            let [value] = values.as_slice() else {
                return None;
            };
            Identities::bounded(namespace, value.split(',').map(str::to_owned).collect())
        });
        Self {
            response_digest: (!body.is_empty()).then(|| {
                Sha256::digest(body)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect()
            }),
            discovery,
            discoveries,
            inputs,
            input_context: T::input_parameter()
                .and_then(|(_, parameter)| request_context(prepared, parameter)),
        }
    }
}

fn request_context(prepared: &Prepared, parameter: &str) -> Option<String> {
    let mut url = url::Url::parse(&prepared.url).ok()?;
    let query: Vec<_> = url
        .query_pairs()
        .filter(|(name, _)| name != parameter)
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect();
    url.set_query(None);
    url.query_pairs_mut().extend_pairs(query);
    let form = match &prepared.post {
        None => Vec::new(),
        Some(post) if post.request_content_type == "application/x-www-form-urlencoded" => {
            url::form_urlencoded::parse(&prepared.body)
                .filter(|(name, _)| name != parameter)
                .map(|(name, value)| (name.into_owned(), value.into_owned()))
                .collect()
        }
        Some(_) => return None,
    };
    let bytes = serde_json::to_vec(&(prepared.post.is_some(), url.as_str(), form)).ok()?;
    Some(
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

impl<T: Reply> crate::read::Read<'_, T> {
    pub(crate) fn provenance(
        &self,
        prepared: &Prepared,
        body: &[u8],
        status: Option<u16>,
    ) -> Provenance {
        let mut provenance = Provenance::received::<T>(prepared, body, status);
        if let Some((inputs, parameter)) = &self.diagnostic_inputs {
            provenance.inputs = Some(inputs.clone());
            provenance.input_context = request_context(prepared, parameter);
        }
        provenance
    }
}
