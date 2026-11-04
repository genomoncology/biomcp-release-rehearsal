//! Keep independently reported component provenance when one native value is invalid.
use crate::native_text::NativeText;
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::Text;
use biomcp_fetch::{BadReply, Endpoint, Freshness, Leg, Reply, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{ComponentUpload, Source, SourceDateTimeValue, VariantComponent};
use serde::Deserialize;
use std::collections::BTreeMap;
const METADATA: Endpoint = super::SPEC.get("/metadata").fresh(Freshness::Days(1));
pub(super) fn cosmic(leg: &Leg<'_>) -> Result<Mapped<Vec<VariantComponent>>, RequestError> {
    components(leg, &[super::components::COSMIC])
}
pub(super) fn components(
    leg: &Leg<'_>,
    selected: &[super::components::Component],
) -> Result<Mapped<Vec<VariantComponent>>, RequestError> {
    let mut raw = leg
        .call(&METADATA, Input::Params(&[]), Read::<Metadata>::json())?
        .value;
    let mut problems = Problems {
        source: Source::MyVariant,
        record: None,
        list: Vec::new(),
    };
    let build = problems.field("build_version", raw.build_version);
    let built = problems.field("build_date", raw.build_date);
    let mut value = Vec::new();
    for declaration in selected {
        let name = Text::new(declaration.name).map_err(|_| super::leg_bad())?;
        problems.record = Some(name.clone());
        let prefix = format!("src.{}", declaration.name);
        let Some(component) = problems.required_field(&prefix, raw.src.remove(declaration.name))
        else {
            continue;
        };
        let upload = problems.field(&format!("{prefix}.upload_date"), component.upload_date);
        let (uploaded, uploads) = uploads(upload, &prefix, &mut problems);
        value.push(VariantComponent {
            terms: super::components::label(declaration.terms)?,
            name,
            version: problems.field(&format!("{prefix}.version"), component.version),
            license: problems.url(&format!("{prefix}.license_url"), component.license_url),
            aggregator_build: build.clone(),
            aggregator_built: built.clone(),
            uploaded,
            uploads,
            note: problems.source_field(&format!("{prefix}.note"), component.note),
        });
    }
    Ok(Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}
fn uploads(
    raw: Option<Uploads>,
    prefix: &str,
    problems: &mut Problems,
) -> (Option<SourceDateTimeValue>, Box<[ComponentUpload]>) {
    match raw {
        None => (None, Box::default()),
        Some(Uploads::One(value)) => (Some(value), Box::default()),
        Some(Uploads::Many(values)) => {
            let values = values
                .into_iter()
                .filter_map(|(name, raw)| {
                    let field = format!("{prefix}.upload_date.{name}");
                    let name = problems.optional(&field, Some(Text::new(&name)))?;
                    let date = problems.field(&field, Some(raw))?;
                    Some(ComponentUpload {
                        name,
                        date: date.into(),
                    })
                })
                .collect();
            (None, values)
        }
    }
}
#[derive(Deserialize)]
struct Metadata {
    build_version: Option<Field<Text>>,
    build_date: Option<Field<SourceDateTimeValue>>,
    src: BTreeMap<String, Field<Component>>,
}
#[derive(Deserialize)]
struct Component {
    version: Option<Field<Text>>,
    license_url: Option<Field<Text>>,
    upload_date: Option<Field<Uploads>>,
    note: Option<Field<NativeText>>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Uploads {
    One(SourceDateTimeValue),
    Many(BTreeMap<String, Field<SourceDateTimeValue>>),
}
impl Reply for Metadata {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
