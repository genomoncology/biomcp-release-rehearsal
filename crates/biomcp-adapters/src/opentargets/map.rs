//! Map native source metadata with field-local disclosure.
use super::raw::{Hit, Indication, Mechanism, Rows};
use crate::{
    Mapped,
    mapped::{Problems, field_text},
    raw::Field,
};
use biodata::{
    Attribution, DrugDiseaseReference, DrugIdentity, DrugIndicationClaim, DrugNativeTarget,
    DrugStage, DrugTargetRelation, Error, SourceGeneRef,
};
use biomcp_types::{DrugHeader, DrugSource, Source};

fn problems(id: &str) -> Problems {
    Problems {
        source: Source::Opentargets,
        record: crate::mapped::record(id),
        list: Vec::new(),
    }
}
fn native_header(
    id: &str,
    name: Option<Field<String>>,
    p: &mut Problems,
) -> Result<DrugHeader, Error> {
    Ok(DrugHeader {
        name_resolution: None,
        matched_name: None,
        id: id.parse()?,
        source: DrugSource::Opentargets,
        name: field_text(&mut p.list, p.source, p.record.clone(), "name", name),
        version: None,
        identity_level: biodata::DrugIdentityLevel::Unknown,
        r#type: Some("drug".parse()?),
        url: Some(format!("https://platform.opentargets.org/drug/{id}").parse()?),
    })
}
/// Preserve the source drug identity independently of its optional name.
pub fn header(row: &Drug) -> Result<Mapped<DrugHeader>, Error> {
    let row = &row.0;
    let mut p = problems(&row.id);
    let name = row.name.as_ref().map(|f| match f.as_ref() {
        Some(s) => Field::Value(s.clone()),
        None => Field::Invalid(serde::de::IgnoredAny),
    });
    let value = native_header(&row.id, name, &mut p)?;
    Ok(Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: p.list,
    })
}
/// Refuse malformed candidate identities while preserving their native page total.
pub fn search_headers(raw: Search, skip: usize) -> Mapped<biomcp_types::DrugPage> {
    let raw = raw.0;
    let mut p = problems("");
    let mut items = Vec::new();
    let mut refusals = crate::Refusal::None;
    for row in raw.hits.into_iter().skip(skip) {
        p.record = None;
        let Some(Hit { id, name, entity }) = p.required_field("identity", Some(row)) else {
            refusals = crate::Refusal::Refused;
            continue;
        };
        p.record = crate::mapped::record(&id);
        if entity != "drug" || !id.starts_with("CHEMBL") {
            p.required::<()>("identity", None);
            refusals = crate::Refusal::Refused;
            continue;
        }
        match native_header(&id, name, &mut p) {
            Ok(row) => items.push(row),
            Err(_) => {
                refusals = crate::Refusal::Refused;
                p.required::<()>("identity", None);
            }
        }
    }
    Mapped {
        refusals,
        value: biomcp_types::DrugPage {
            items: items.into(),
            total: Some(raw.total),
        },
        problems: p.list,
    }
}
/// Preserve each admitted native target while omitting rejected mechanism metadata.
pub fn targets(
    drug: &DrugIdentity,
    rows: Mechanisms,
) -> Result<Mapped<Box<[DrugTargetRelation]>>, Error> {
    let mut p = problems(&drug.id.to_string());
    let mut values = Vec::new();
    let mut refusals = crate::Refusal::None;
    for row in rows.0.rows {
        let Some(row) = p.required_field("mechanismsOfAction", Some(row)) else {
            refusals = crate::Refusal::Refused;
            continue;
        };
        let mechanism = field_text(
            &mut p.list,
            p.source,
            p.record.clone(),
            "mechanismOfAction",
            row.mechanism_of_action,
        );
        // Existing source mapping policy uses a generic relation only when actionType is absent.
        let relation = match row.action_type {
            None => Some("mechanism".parse()?),
            value => field_text(&mut p.list, p.source, p.record.clone(), "actionType", value),
        };
        for target in p.fields("targets", row.targets) {
            let Some(id) = p.optional(
                "targets.id",
                Some(target.id.parse::<biodata::EnsemblGeneId>()),
            ) else {
                refusals = crate::Refusal::Refused;
                continue;
            };
            let label = field_text(
                &mut p.list,
                p.source,
                p.record.clone(),
                "targets.approvedName",
                target.approved_name,
            );
            let symbol = crate::mapped::field_code_text(
                &mut p.list,
                p.source,
                p.record.clone(),
                "targets.approvedSymbol",
                target.approved_symbol,
            );
            let gene = symbol
                .map(|label| SourceGeneRef::new(label, None, None, Some(id.clone())))
                .transpose()?;
            let native = DrugNativeTarget {
                namespace: "opentargets".parse()?,
                id: target.id.parse()?,
                label,
                url: None,
            };
            values.push(DrugTargetRelation::new(
                drug.clone(),
                gene,
                Some(native),
                relation.clone(),
                mechanism.clone(),
                Box::default(),
                attribution(drug)?,
            )?);
        }
    }
    Ok(Mapped {
        refusals,
        value: values.into(),
        problems: p.list,
    })
}
/// Preserve disease identities independently of optional indication labels and stages.
pub fn indications(
    drug: &DrugIdentity,
    rows: Indications,
) -> Result<Mapped<Box<[DrugIndicationClaim]>>, Error> {
    let mut p = problems(&drug.id.to_string());
    let mut values = Vec::new();
    let mut refusals = crate::Refusal::None;
    for row in rows.0.rows {
        let Some(row) = p.required_field("indications", Some(row)) else {
            refusals = crate::Refusal::Refused;
            continue;
        };
        let Some(id) = p.optional("disease.id", Some(row.disease.id.parse::<biodata::Text>()))
        else {
            refusals = crate::Refusal::Refused;
            continue;
        };
        let label = field_text(
            &mut p.list,
            p.source,
            p.record.clone(),
            "disease.name",
            row.disease.name,
        );
        let stage = p.field("maxClinicalStage", row.max_clinical_stage);
        let disease = DrugDiseaseReference {
            namespace: row
                .disease
                .id
                .split('_')
                .next()
                .unwrap_or("opentargets")
                .parse()?,
            id,
            label: label.clone(),
        };
        values.push(DrugIndicationClaim {
            drug: drug.clone(),
            indication: label,
            disease_references: Box::new([disease]),
            stage: stage.map(|value| DrugStage {
                field: "maxClinicalStage".into(),
                value: value.into(),
            }),
            references: Box::default(),
            population: None,
            combination: Box::default(),
            attribution: attribution(drug)?,
        });
    }
    Ok(Mapped {
        refusals,
        value: values.into(),
        problems: p.list,
    })
}
fn attribution(drug: &DrugIdentity) -> Result<Attribution, Error> {
    Ok(Attribution {
        approved: None,
        published: None,
        source: "opentargets".parse()?,
        record: Some(drug.id.to_string().parse()?),
        version: None,
        release: None,
        submitter: None,
        evaluated: None,
        url: None,
    })
}

/// Accepted source search with private provider fields.
#[derive(Debug)]
pub struct Search(pub(super) super::raw::Search);
impl Search {
    /// Native rows consumed before malformed candidate identities are refused.
    pub fn received(&self) -> usize {
        self.0.hits.len()
    }
}
/// Accepted source drug with private provider fields.
#[derive(Debug)]
pub struct Drug(pub(super) super::raw::Drug);
/// Native mechanism collection retained privately until mapping.
#[derive(Debug, serde::Deserialize)]
#[serde(transparent)]
pub struct Mechanisms(Rows<Mechanism>);
/// Native indication collection retained privately until mapping.
#[derive(Debug, serde::Deserialize)]
#[serde(transparent)]
pub struct Indications(Rows<Indication>);
impl Drug {
    /// Select the source's original complete mechanism collection.
    pub fn mechanisms(self) -> Mechanisms {
        Mechanisms(self.0.mechanisms_of_action)
    }
    /// Select the source's original complete indication collection.
    pub fn indications(self) -> Indications {
        Indications(self.0.indications)
    }
}

/// Map the native source card and supported sections.
pub fn card(header: DrugHeader, fetched: Option<i64>) -> biomcp_types::DrugCard {
    biomcp_types::DrugCard {
        header,
        alternate_names: Box::default(),
        cross_references: Box::default(),
        parents: Box::default(),
        products: Box::default(),
        formulations: Box::default(),
        version: None,
        updated: None,
        retrieved: crate::drug_sources::received_date(fetched),
        available_sections: Box::new([
            biomcp_types::DrugSection::Targets,
            biomcp_types::DrugSection::Indications,
        ]),
        next: Box::default(),
    }
}
