use super::raw::{Object, Relation};
use biodata::{
    GtrConditionReference, GtrConditionRelation, GtrGeneReference, GtrGeneRelation, GtrMethod,
    GtrMethodList, GtrTestRelation, GtrTestVersion, Text,
};
use biomcp_fetch::RequestError;
use biomcp_types::{DiagnosticLaboratory, DiagnosticSection, GtrCard, GtrHeader, GtrNativeStatus};
fn optional(
    value: &str,
    field: &str,
    v: &GtrTestVersion,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Option<Text> {
    // The retained GTR report defines empty native cells as absent (diagnostic slice field review).
    if value.is_empty() {
        return None;
    }
    crate::mapped::field_text(
        problems,
        biomcp_types::Source::Gtr,
        crate::mapped::record(&v.test_accession_ver),
        field,
        Some(crate::raw::Field::Value(value.to_owned())),
    )
}
pub(super) fn header(
    v: &GtrTestVersion,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Result<GtrHeader, RequestError> {
    // The publisher's web route uses the numeric test base; the typed ID keeps
    // the complete accession/version independently of this current listing link.
    let base = v
        .test_accession_ver
        .as_str()
        .strip_prefix("GTR")
        .and_then(|id| id.split_once('.'))
        .ok_or_else(|| biomcp_fetch::bad_reply(biomcp_types::Source::Gtr, None))?
        .0;
    let base: u32 = base
        .parse()
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Gtr, None))?;
    Ok(GtrHeader {
        id: v.test_accession_ver.clone(),
        name: optional(&v.lab_test_name, "lab_test_name", v, problems),
        laboratory: crate::mapped::field_code_text(
            problems,
            biomcp_types::Source::Gtr,
            crate::mapped::record(&v.test_accession_ver),
            "lab_id",
            (!v.lab_id.is_empty()).then(|| crate::raw::Field::Value(v.lab_id.to_string())),
        )
        .map(|id| DiagnosticLaboratory {
            id,
            name: optional(&v.name_of_laboratory, "name_of_laboratory", v, problems),
            institution: optional(&v.name_of_institution, "name_of_institution", v, problems),
            country: optional(&v.facility_country, "facility_country", v, problems),
        }),
        url: Text::new(&format!(
            "https://www.ncbi.nlm.nih.gov/gtr/tests/classic/{base}/"
        ))
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Gtr, None))?,
    })
}
pub(super) fn card(
    v: &GtrTestVersion,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Result<GtrCard, RequestError> {
    Ok(GtrCard {
        header: header(v, problems)?,
        native_status: GtrNativeStatus {
            now_current: v.now_current.clone(),
            test_current: v.test_current.clone(),
            test_public: v.test_public.clone(),
            laboratory_current: v.laboratory_current.clone(),
            laboratory_public: v.laboratory_public.clone(),
        },
        last_touch_date: optional(&v.last_touch_date, "last_touch_date", v, problems),
        test_create_date: optional(&v.test_create_date, "test_create_date", v, problems),
        test_deletion_date: v.test_deletion_date.clone(),
        manufacturer_test_name: v.manufacturer_test_name.clone(),
        test_development: v.test_development.clone(),
        lab_test_id: v.lab_test_id.clone(),
        test_order_code: v.test_order_code.clone(),
        clia_number: v.clia_number.clone(),
        state_licenses: v.state_licenses.clone(),
        state_license_numbers: v.state_license_numbers.clone(),
        sections: vec![
            DiagnosticSection::Genes,
            DiagnosticSection::Conditions,
            DiagnosticSection::Methods,
        ]
        .into(),
    })
}
pub(super) fn relation(
    r: &Relation,
    cx: &mut crate::mapped::Problems,
    refusals: &mut crate::Refusal,
) -> Option<GtrTestRelation> {
    cx.record = crate::mapped::record(&r.id);
    let test_type = cx.optional(
        "test_type",
        r.test_type
            .as_ref()
            .map(|value| value.as_ref().copied().ok_or(())),
    );
    let medgen = cx.optional(
        "GTR_identifier",
        (!r.medgen.is_empty()).then(|| format!("MedGen:{}", r.medgen).parse()),
    );
    if r.object == Object::Gene {
        // A non-GeneID cell such as native -1 remains literal context, not a converted identifier.
        let gene_id = r.object_id.parse::<biodata::NcbiGeneId>().ok();
        let label = cx.checked_text(
            "gene_symbol",
            (!r.gene_symbol.is_empty()).then_some(&r.gene_symbol),
        );
        if gene_id.is_none() && label.is_none() {
            *refusals = crate::Refusal::Refused;
            cx.required::<()>("gene_symbol", None);
            return None;
        }
        Some(GtrTestRelation::Gene(GtrGeneRelation {
            test_id: r.id.clone(),
            test_type,
            gene: GtrGeneReference {
                label,
                ncbi_gene_id: gene_id,
            },
            native_gene_or_snomed_ct_id: (!r.object_id.is_empty() && gene_id.is_none())
                .then(|| r.object_id.clone()),
            native_medgen: medgen,
            mim_number: r.mim.clone(),
            object_name: r.object_name.clone(),
        }))
    } else {
        let Some(id) = cx.required("GTR_identifier", medgen) else {
            *refusals = crate::Refusal::Refused;
            return None;
        };
        let label = cx.text(
            "object_name",
            (!r.object_name.is_empty()).then_some(&r.object_name),
        );
        Some(GtrTestRelation::Condition(GtrConditionRelation {
            test_id: r.id.clone(),
            test_type,
            condition: GtrConditionReference { id, label },
            mim_number: r.mim.clone(),
            snomed_ct_id: r.object_id.clone(),
        }))
    }
}
pub(super) fn methods(v: &GtrTestVersion) -> Vec<GtrMethod> {
    [
        (GtrMethodList::Categories, &v.method_categories),
        (GtrMethodList::Methods, &v.methods),
        (GtrMethodList::Platforms, &v.platforms),
    ]
    .into_iter()
    .flat_map(|(list, cell)| {
        cell.split('|')
            .enumerate()
            .filter(|(_, value)| !value.is_empty())
            .map(move |(index, value)| GtrMethod {
                test_id: v.test_accession_ver.clone(),
                list,
                index: index as u32,
                value: value.into(),
            })
    })
    .collect()
}
