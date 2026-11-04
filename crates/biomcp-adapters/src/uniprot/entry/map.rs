use super::{Entry, raw};
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::{Protein, ProteinAudit, ProteinHeader, Text, UniProtIsoform};
use biomcp_types::Source;

pub(in crate::uniprot) fn description(
    raw: Option<Field<raw::Description>>,
    p: &mut Problems,
) -> Option<Text> {
    let raw = p.field("protein_name", raw)?;
    let name = p.field("protein_name", raw.recommended_name)?;
    let value = p.field("protein_name", name.full_name)?;
    p.source_field("protein_name", value.value)
}
pub(in crate::uniprot) fn length(raw: &raw::Sequence, p: &mut Problems) -> Option<u32> {
    p.optional(
        "length",
        raw.length
            .as_ref()
            .map(|v| v.as_ref().copied().filter(|n| *n > 0).ok_or(())),
    )
}
fn audit(raw: Option<Field<raw::Audit>>, p: &mut Problems) -> ProteinAudit {
    let mut value = ProteinAudit {
        entry_version: None,
        sequence_version: None,
        first_public: None,
        last_annotation_update: None,
        last_sequence_update: None,
        release: None,
    };
    if let Some(raw) = p.field("entryAudit", raw) {
        value.entry_version = p.field("entryAudit.entryVersion", raw.entry_version);
        value.sequence_version = p.field("entryAudit.sequenceVersion", raw.sequence_version);
        value.first_public = p.field("entryAudit.firstPublicDate", raw.first_public_date);
        value.last_annotation_update = p.field(
            "entryAudit.lastAnnotationUpdateDate",
            raw.last_annotation_update_date,
        );
        value.last_sequence_update = p.field(
            "entryAudit.lastSequenceUpdateDate",
            raw.last_sequence_update_date,
        );
    }
    value
}
fn sequence(
    raw: Option<raw::Sequence>,
    id: &UniProtIsoform,
    length: Option<u32>,
    audit: &ProteinAudit,
    p: &mut Problems,
) -> Option<biodata::ProteinSequence> {
    let raw = raw?;
    let residues = p.field("sequence.value", raw.value);
    let crc64 = p.source_field("sequence.crc64", raw.crc64);
    let md5 = p.source_field("sequence.md5", raw.md5);
    let length = length?;
    let residues = p.optional(
        "sequence.value",
        residues.map(|r| {
            if r.as_str().len() == length as usize {
                Ok(r)
            } else {
                Err(())
            }
        }),
    )?;
    Some(biodata::ProteinSequence {
        accession: id.clone(),
        sequence: residues,
        length,
        crc64,
        md5,
        audit: audit.clone(),
    })
}
fn gene_names(raw: Option<Field<Vec<Field<raw::Gene>>>>, p: &mut Problems) -> Box<[Text]> {
    let mut names = Vec::new();
    for gene in p.fields("genes", raw) {
        if let Some(value) = p.field("genes.geneName", gene.gene_name) {
            names.extend(p.source_field("genes.geneName", value.value));
        }
        for (field, values) in [
            ("genes.synonyms", gene.synonyms),
            ("genes.orderedLocusNames", gene.ordered_locus_names),
            ("genes.orfNames", gene.orf_names),
        ] {
            for value in p.fields(field, values) {
                names.extend(p.source_field(field, value.value));
            }
        }
    }
    names.into()
}
pub(in crate::uniprot) fn entry(raw: raw::Entry, id: &UniProtIsoform) -> Mapped<Option<Entry>> {
    let mut p = Problems {
        source: Source::UniProt,
        record: crate::mapped::record(id),
        list: Vec::new(),
    };
    if !raw.admitted(id) {
        p.required::<()>(
            if raw.primary_accession != *id {
                "primaryAccession"
            } else {
                "reviewed_human"
            },
            None,
        );
        return Mapped {
            refusals: crate::Refusal::Refused,
            value: None,
            problems: p.list,
        };
    }
    let audit = audit(raw.entry_audit, &mut p);
    let feature_rows = p.fields("features", raw.features);
    let features = super::sections_map::features(feature_rows.into_vec(), id, &audit, &mut p);
    let comment_rows = p.fields("comments", raw.comments);
    let (isoforms, function) = super::sections_map::comments(comment_rows.into_vec(), id, &mut p);
    let reference_rows = p.fields("uniProtKBCrossReferences", raw.uni_prot_kb_cross_references);
    let structures = super::sections_map::structures(reference_rows.into_vec(), &mut p);
    let seq = p.field("sequence", raw.sequence);
    let length = seq.as_ref().and_then(|seq| length(seq, &mut p));
    let sequence = sequence(seq, id, length, &audit, &mut p);
    let name = description(raw.protein_description, &mut p);
    let entry_id = p.source_field("uniProtkbId", raw.uni_protkb_id);
    let gene_names = gene_names(raw.genes, &mut p);
    let url = crate::mapped::text(&format!("https://www.uniprot.org/uniprotkb/{id}/entry"));
    let value = p.optional("url", Some(url)).map(|url| Entry {
        header: ProteinHeader {
            protein: Protein {
                accession: raw.primary_accession,
                name,
                length,
                function: Box::default(),
                subcellular_locations: Box::default(),
            },
            entry_id,
            gene_names,
            gene_ids: Box::default(),
            taxon: 9606,
            reviewed: true,
            url,
        },
        audit,
        features,
        isoforms,
        function,
        structures,
        sequence,
    });
    Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: p.list,
    }
}
