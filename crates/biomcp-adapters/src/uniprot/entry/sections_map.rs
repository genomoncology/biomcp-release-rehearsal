use super::sections_raw as raw;
use crate::mapped::Problems;
use biodata::{
    Attribution, Bound, ProteinAudit, ProteinCoordinateConvention, ProteinFeature, ProteinFunction,
    ProteinIsoform, ProteinSequenceAvailability, ProteinStructureRef, Term, Text, UncertainSpan0,
    UniProtFeature, UniProtIsoform,
};

pub(super) fn evidence(rows: Vec<raw::Evidence>, p: &mut Problems) -> Box<[Text]> {
    rows.into_iter()
        .filter_map(|e| {
            let mut text = e.evidence_code.to_string();
            if let Some(source) = p.source_field("evidence.source", e.source) {
                text.push_str(&format!("|{source}"));
            }
            if let Some(id) = p.source_field("evidence.id", e.id) {
                text.push_str(&format!("|{id}"));
            }
            p.text("evidence", Some(&text))
        })
        .collect()
}
enum Side {
    Start,
    End,
}
fn bound(v: raw::Boundary, side: Side) -> Option<Bound> {
    if matches!(v.modifier, raw::Modifier::Unknown | raw::Modifier::Unsure) {
        return Bound::new(Some(1), None).ok();
    }
    let n = v.value?;
    if n == 0 {
        return None;
    }
    match v.modifier {
        raw::Modifier::Exact => Bound::new(Some(n), Some(n)).ok(),
        raw::Modifier::Outside if matches!(side, Side::Start) => Bound::new(None, Some(n)).ok(),
        raw::Modifier::Outside => Bound::new(Some(n), None).ok(),
        _ => None,
    }
}

pub(super) fn features(
    rows: Vec<raw::Feature>,
    id: &UniProtIsoform,
    audit: &ProteinAudit,
    p: &mut Problems,
) -> Vec<ProteinFeature> {
    rows.into_iter()
        .filter_map(|r| {
            let span = bound(r.location.start, Side::Start)
                .zip(bound(r.location.end, Side::End))
                .and_then(|(start, end)| {
                    p.optional(
                        "features.location",
                        Some(UncertainSpan0::from_inclusive(start, end)),
                    )
                });
            let span = p.required("features.location", span)?;
            let alternative = p.field("features.alternativeSequence", r.alternative_sequence);
            let (original, alternative) = alternative.map_or_else(
                || (None, Box::default()),
                |a| {
                    (
                        p.field("features.originalSequence", a.original_sequence),
                        p.fields("features.alternativeSequences", a.alternative_sequences),
                    )
                },
            );
            let evidence_rows = p.fields("evidence", r.evidences);
            Some(ProteinFeature {
                feature: UniProtFeature {
                    protein: r.location.sequence.unwrap_or_else(|| id.clone()),
                    span,
                    kind: Term {
                        label: r.kind,
                        codings: Box::default(),
                    },
                    description: crate::mapped::field_text(
                        &mut p.list,
                        p.source,
                        p.record.clone(),
                        "features.description",
                        r.description,
                    ),
                    id: p.source_field("features.featureId", r.feature_id),
                    original,
                    alternative: alternative.into_vec(),
                    evidence: evidence(evidence_rows.into_vec(), p).into_vec(),
                },
                attribution: Attribution {
                    approved: None,
                    published: None,
                    source: p
                        .optional("attribution.source", Some(crate::mapped::text("uniprot")))?,
                    record: crate::mapped::record(id),
                    version: p.optional(
                        "entryAudit.entryVersion",
                        audit
                            .entry_version
                            .map(|v| crate::mapped::text(&v.to_string())),
                    ),
                    release: audit.release.clone(),
                    submitter: None,
                    evaluated: audit.last_annotation_update,
                    url: p.optional(
                        "attribution.url",
                        Some(crate::mapped::text(&format!(
                            "https://www.uniprot.org/uniprotkb/{id}/entry"
                        ))),
                    ),
                },
                source_coordinates: ProteinCoordinateConvention::OneBasedInclusive,
            })
        })
        .collect()
}
pub(super) fn comments(
    rows: Vec<raw::Comment>,
    _id: &UniProtIsoform,
    p: &mut Problems,
) -> (Vec<ProteinIsoform>, Vec<ProteinFunction>) {
    let mut isoforms = Vec::new();
    let mut function = Vec::new();
    for row in rows {
        if row.comment_type == "ALTERNATIVE PRODUCTS" {
            for r in p.fields("isoforms", row.isoforms) {
                let name = p
                    .field("isoforms.name", r.name)
                    .and_then(|n| p.source_field("isoforms.name", n.value));
                let synonyms = p
                    .fields("isoforms.synonyms", r.synonyms)
                    .into_iter()
                    .filter_map(|v| p.source_field("isoforms.synonyms", v.value))
                    .collect();
                let sequence_ids = p
                    .fields("isoforms.sequenceIds", r.sequence_ids)
                    .into_iter()
                    .map(Into::into)
                    .collect();
                isoforms.push(ProteinIsoform {
                    ids: p.fields("isoforms.isoformIds", r.isoform_ids),
                    name,
                    synonyms,
                    sequence_status: p
                        .field("isoforms.isoformSequenceStatus", r.isoform_sequence_status),
                    sequence_ids,
                    direct_sequence: ProteinSequenceAvailability::NotChecked,
                });
            }
        } else if row.comment_type == "FUNCTION" {
            let molecule = p.source_field("function.molecule", row.molecule);
            for paragraph in p.fields("function", row.texts) {
                let rows = p.fields("evidence", paragraph.evidences);
                function.push(ProteinFunction {
                    molecule: molecule.clone(),
                    text: paragraph.value,
                    evidence: evidence(rows.into_vec(), p),
                });
            }
        }
    }
    (isoforms, function)
}
pub(super) fn structures(
    rows: Vec<raw::CrossReference>,
    p: &mut Problems,
) -> Vec<ProteinStructureRef> {
    rows.into_iter()
        .filter(|r| r.database.as_str() == "PDB")
        .map(|r| ProteinStructureRef {
            url: p.optional(
                "structures.url",
                Some(crate::mapped::text(&format!(
                    "https://www.rcsb.org/structure/{}",
                    r.id
                ))),
            ),
            namespace: r.database,
            id: r.id,
            properties: p.fields("structures.properties", r.properties),
        })
        .collect()
}
