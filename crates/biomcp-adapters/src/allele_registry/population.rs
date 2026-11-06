//! Preserve the registry's gnomAD v4 cross-reference, never infer padding or alignment.
use crate::{Mapped, mapped::Problems};
use biodata::Text;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::Source;
use biomcp_types::VariantInput;

/// Read one canonical allele's exact GRCh38 gnomAD v4 key.
/// gnomAD v2 uses GRCh37 and is deliberately not a fallback.
pub fn population_id(leg: &Leg<'_>, id: &str) -> Result<Mapped<Option<Text>>, RequestError> {
    let raw = super::read_fields(leg, id, true, "none+@id+externalRecords.gnomAD_4")?;
    raw.identity(Some(id))
        .map_err(|_| biomcp_fetch::bad_reply(Source::AlleleRegistry, None))?;
    let mut fields = Problems {
        source: Source::AlleleRegistry,
        record: Some(
            Text::new(id).map_err(|_| biomcp_fetch::bad_reply(Source::AlleleRegistry, None))?,
        ),
        list: Vec::new(),
    };
    let mut ids = std::collections::HashSet::new();
    if let Some(external) = fields.field("externalRecords", raw.external) {
        for row in fields.fields("externalRecords.gnomAD_4", external.gnomad) {
            let id = fields.field("externalRecords.gnomAD_4.id", Some(row.id));
            let parsed = fields.optional(
                "externalRecords.gnomAD_4.id",
                id.as_ref().map(|id| id.as_str().parse::<VariantInput>()),
            );
            if let Some((id, parsed)) = id.zip(parsed)
                && fields
                    .required(
                        "externalRecords.gnomAD_4.id",
                        parsed.coordinate_form().then_some(()),
                    )
                    .is_some()
            {
                ids.insert(id);
            }
        }
    }
    let value = if ids.len() == 1 && fields.list.is_empty() {
        ids.into_iter().next()
    } else {
        if ids.len() > 1 {
            fields.required::<()>("externalRecords.gnomAD_4.id", None);
        }
        None
    };
    Ok(Mapped {
        value,
        problems: fields.list,
        refusals: crate::Refusal::None,
    })
}
