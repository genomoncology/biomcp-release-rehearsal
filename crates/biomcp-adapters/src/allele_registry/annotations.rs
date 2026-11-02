//! Source annotations never determine admission of the canonical CA identity.
use super::raw;
use crate::{mapped::Problems, raw::Field};
use biodata::{Expression, Representation, VariantId};
use biomcp_types::Source;
pub(super) fn ids(raw: Option<Field<raw::External>>, fields: &mut Problems) -> Vec<VariantId> {
    let mut ids = Vec::new();
    if let Some(external) = fields.field("externalRecords", raw) {
        for rs in fields.fields("externalRecords.dbSNP.rs", external.rs) {
            if let Some(id) =
                fields.optional("externalRecords.dbSNP.rs", Some(biodata::RsId::new(rs.rs)))
            {
                ids.push(VariantId::Rs(id));
            }
        }
        for cv in fields.fields(
            "externalRecords.ClinVarVariations.variationId",
            external.clinvar,
        ) {
            if let Some(id) = fields.optional(
                "externalRecords.ClinVarVariations.variationId",
                Some(
                    u32::try_from(cv.id)
                        .map_err(|_| biodata::Error::ClinVarVariationIdInvalid)
                        .and_then(|id| {
                            biodata::ClinVarVariationId::new(id)
                                .map_err(|_| biodata::Error::ClinVarVariationIdInvalid)
                        }),
                ),
            ) {
                ids.push(VariantId::ClinVarVariation(id));
            }
        }
    }
    ids
}
pub(super) fn forms(
    raw: Option<Field<Vec<Field<raw::Transcript>>>>,
    fields: &mut Problems,
) -> Vec<Representation> {
    let mut representations = Vec::new();
    for t in fields.fields("transcriptAlleles", raw) {
        if let Some(m) = fields.field("transcriptAlleles.MANE", t.mane) {
            for forms in [m.nucleotide, m.protein].into_iter().flatten() {
                if let Some(forms) = fields.field("transcriptAlleles.MANE", Some(forms)) {
                    for form in [forms.ensembl, forms.refseq].into_iter().flatten() {
                        let form = fields.field("transcriptAlleles.MANE.hgvs", Some(form));
                        let hgvs = form.and_then(|form| {
                            fields.field("transcriptAlleles.MANE.hgvs", Some(form.hgvs))
                        });
                        if let Some(expression) = fields
                            .optional("transcriptAlleles.MANE.hgvs", hgvs.map(Expression::hgvs))
                        {
                            representations.push(Representation {
                                source: Source::AlleleRegistry.text(),
                                expression,
                                build: None,
                                allele: None,
                            });
                        }
                    }
                }
            }
        }
    }
    representations
}
