//! One publisher-documented per-pathway GPML request, with no catalog fallback.
use super::{
    SPEC,
    gpml_raw::{Document, MAX_BYTES},
};
use crate::Mapped;
use crate::mapped::{Problems, record};
use biodata::{PathwayMember, PathwaySourceRef, Text, WikiPathwaysId};
use biomcp_fetch::{Freshness, Leg, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::Source;
use std::collections::BTreeSet;

/// Complete GeneProduct nodes and observed source node counts, before paging.
#[derive(Debug)]
pub struct Members {
    /// Whole native GeneProduct data nodes, including repeated references.
    pub items: Vec<PathwayMember>,
    /// GPML's own full WP identifier and r revision, never the catalog's date.
    pub version: Text,
    /// GPML's source-stated pathway name.
    pub name: Option<Text>,
    /// All DataNodes, including excluded Metabolite and Pathway nodes.
    pub data_nodes: u64,
}
fn invalid() -> RequestError {
    biomcp_fetch::SourceError {
        source: Source::WikiPathways,
        reason: biomcp_types::Reason::BadReply,
        status: Some(200),
        tls_cause: None,
        retry_after: None,
    }
    .into()
}

/// Fetch exactly the publisher's documented GPML locator and validate identity before caching.
pub fn members(leg: &Leg<'_>, id: &WikiPathwaysId) -> Result<Mapped<Members>, RequestError> {
    let endpoint = SPEC
        .get("/wikipathways-assets/pathways")
        .with_path(format!("/wikipathways-assets/pathways/{id}/{id}.gpml"))?
        .once()
        .application_octet_stream()
        .bytes(MAX_BYTES)
        .fresh(Freshness::Days(7));
    let (document, _) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            Read::<Document>::xml().check(&|value| value.matches(id)),
        )
        .map(|received| (received.value, received.meta))?;
    let mut p = Problems {
        source: Source::WikiPathways,
        record: record(id),
        list: Vec::new(),
    };
    let mut items = Vec::new();
    let mut ids = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for node in &document.nodes {
        if let Some(id) = &node.id
            && !ids.insert(id.clone())
        {
            duplicates.insert(id.clone());
        }
    }
    let data_nodes = document.nodes.len() as u64;
    let mut refusals = crate::Refusal::None;
    for node in document.nodes {
        if let Some(member) = member(node, &duplicates, &mut p, &mut refusals)? {
            items.push(member);
        }
    }
    let name = p.text("name", document.name.as_deref());
    Ok(Mapped {
        refusals,
        value: Members {
            items,
            version: document.version.parse().map_err(|_| invalid())?,
            name,
            data_nodes,
        },
        problems: p.list,
    })
}

fn member(
    node: super::gpml_raw::Node,
    duplicates: &BTreeSet<String>,
    p: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Result<Option<PathwayMember>, RequestError> {
    if node.kind.as_deref() != Some("GeneProduct") {
        if node.namespace.as_deref().is_none_or(str::is_empty) {
            p.required::<()>(
                &format!(
                    "data_nodes.{}.xref",
                    node.kind.as_deref().unwrap_or("unknown")
                ),
                None,
            );
        }
        return Ok(None);
    }
    let node_id = p.optional(
        "genes.node_id",
        node.id.as_deref().map(|v| {
            Text::new(v)
                .map_err(|_| ())
                .and_then(|t| if t.as_str() == v { Ok(t) } else { Err(()) })
        }),
    );
    if node.id.is_some() && node_id.is_none() {
        *refusals = crate::Refusal::Refused;
        return Ok(None);
    }
    if node.id.as_ref().is_some_and(|id| duplicates.contains(id)) {
        p.required::<()>("genes.node_id", None);
        *refusals = crate::Refusal::Refused;
        return Ok(None);
    }
    let value = p.optional(
        "genes.reference",
        node.value.as_deref().map(|v| {
            Text::new(v)
                .map_err(|_| ())
                .and_then(|t| if t.as_str() == v { Ok(t) } else { Err(()) })
        }),
    );
    let Some(value) = p.required("genes.reference", value) else {
        *refusals = crate::Refusal::Refused;
        return Ok(None);
    };
    if node.namespace.as_deref().is_none_or(str::is_empty) {
        p.required::<()>("genes.reference", None);
    }
    Ok(Some(PathwayMember {
        reference: PathwaySourceRef {
            namespace: node
                .namespace
                .as_deref()
                .and_then(|value| p.text("genes.namespace", Some(value)))
                .into(),
            id: value,
        },
        node_id,
        name: p.text("genes.name", node.label.as_deref()),
        source_url: None,
        gene: None,
        schema_class: Some("GeneProduct".parse().map_err(|_| invalid())?),
        physical_context_count: 1,
        participants: None,
    }))
}
