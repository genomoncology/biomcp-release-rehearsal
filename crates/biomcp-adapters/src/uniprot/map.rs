use super::{entry, raw};
use crate::{
    Mapped,
    mapped::{Problems, push_problem, record},
    raw::Field,
};
use biodata::{Text, UniProtAcc};
use biomcp_types::{Protein, Source};
pub(super) fn proteins(raw: raw::Proteins, ids: &[UniProtAcc]) -> Mapped<Vec<Protein>> {
    let mut problems = Vec::new();
    let mut values = Vec::new();
    let mut seen = Vec::new();
    let mut refusals = crate::Refusal::None;
    for raw in raw.results {
        let Field::Value(raw) = raw else {
            push_problem(&mut problems, Source::UniProt, None, "protein");
            refusals = crate::Refusal::Refused;
            continue;
        };
        let id = match raw.primary_accession.parse::<UniProtAcc>() {
            Ok(id) => id,
            Err(_) => {
                push_problem(&mut problems, Source::UniProt, None, "primaryAccession");
                refusals = crate::Refusal::Refused;
                continue;
            }
        };
        if !ids.contains(&id) || seen.contains(&id) {
            push_problem(
                &mut problems,
                Source::UniProt,
                record(&id),
                "primaryAccession",
            );
            values.retain(|value: &Protein| value.accession != id);
            refusals = crate::Refusal::Refused;
            continue;
        }
        seen.push(id.clone());
        if raw.organism.taxon_id != 9606 || raw.entry_type != "UniProtKB reviewed (Swiss-Prot)" {
            push_problem(
                &mut problems,
                Source::UniProt,
                record(&id),
                "reviewed_human",
            );
            refusals = crate::Refusal::Refused;
            continue;
        }
        let mut p = Problems {
            source: Source::UniProt,
            record: record(&id),
            list: Vec::new(),
        };
        let name = entry::map::description(raw.protein_description, &mut p);
        let seq = p.field("sequence", raw.sequence);
        let length = seq.as_ref().and_then(|seq| entry::map::length(seq, &mut p));
        let (function, subcellular_locations) = comments(raw.comments, &mut p);
        values.push(Protein {
            accession: id,
            name,
            length,
            function: function.into(),
            subcellular_locations: subcellular_locations.into(),
        });
        problems.extend(p.list);
    }
    for id in ids {
        if !seen.contains(id) {
            push_problem(&mut problems, Source::UniProt, record(id), "missing");
        }
    }
    Mapped {
        refusals,
        value: values,
        problems,
    }
}
fn comments(
    raw: Option<Field<Vec<Field<raw::Comment>>>>,
    p: &mut Problems,
) -> (Vec<Text>, Vec<Text>) {
    let mut function = Vec::new();
    let mut locations = Vec::new();
    for comment in p.fields("comments", raw) {
        match comment.comment_type.as_str() {
            "FUNCTION" => {
                for value in p.fields("function", comment.texts) {
                    function.extend(p.source_field("function", value.value));
                }
            }
            "SUBCELLULAR LOCATION" => {
                for value in p.fields("subcellular_locations", comment.subcellular_locations) {
                    locations.extend(p.source_field("subcellular_locations", value.location.value));
                }
            }
            _ => {}
        }
    }
    (function, locations)
}
