//! Capture-proven native measurements, without score blending or interpretation.
use biomcp_fetch::Input;
mod map;
mod raw;
use biodata::Build;
use biomcp_fetch::{Arg, Leg, PathSegment, RequestError};
use biomcp_types::VariantPredictors;
/// Read the proven prediction or conservation component projection for one exact parent.
pub fn get(
    leg: &Leg<'_>,
    id: &str,
    build: Build,
    conservation: bool,
    cadd: bool,
    dbnsfp: bool,
) -> Result<crate::Mapped<VariantPredictors>, RequestError> {
    let mut fields = vec!["_id"];
    if cadd {
        fields.extend(["cadd.phred", "cadd.rawscore", "cadd.sift"]);
    }
    if dbnsfp {
        fields.extend(["dbnsfp.hgvsp", "dbnsfp.ensembl.transcriptid"]);
        if conservation {
            fields.extend([
                "dbnsfp.gerp++",
                "dbnsfp.phastcons",
                "dbnsfp.phylop",
                "dbnsfp.siphy_29way",
            ]);
        } else {
            fields.extend(["dbnsfp.polyphen2", "dbnsfp.revel", "dbnsfp.mutationtaster"]);
        }
    }
    let fields = fields.join(",");
    let endpoint = super::VARIANT
        .clone()
        .with_segments(&[PathSegment::new(id)?]);
    let record = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("fields", Arg::Text(&fields)),
                (
                    "assembly",
                    Arg::Text(if build == Build::GRCh38 {
                        "hg38"
                    } else {
                        "hg19"
                    }),
                ),
            ]),
            biomcp_fetch::Read::<raw::Record>::json().cache_if(&|record| {
                record
                    .id
                    .as_ref()
                    .and_then(crate::raw::Field::as_ref)
                    .is_some_and(|given| given.as_str() == id)
            }),
        )?
        .value;
    let mut mapped = map::project(record, id, build, conservation, cadd, dbnsfp)?;
    let selected = [
        (cadd, super::components::CADD),
        (dbnsfp, super::components::DBNSFP),
    ]
    .into_iter()
    .filter_map(|(enabled, component)| enabled.then_some(component))
    .collect::<Vec<_>>();
    if !mapped.value.scores.is_empty() || !mapped.value.context.is_empty() {
        let metadata = super::metadata::components(leg, &selected)?;
        mapped.problems.extend(metadata.problems);
        mapped.value.components = metadata.value.into();
    }
    for score in &mut mapped.value.scores {
        if let Some(component) = mapped.value.components.iter().find(|component| {
            score
                .method
                .as_str()
                .starts_with(&format!("{}.", component.name.as_str()))
        }) {
            score.attribution.release = component.version.clone();
        }
    }
    Ok(mapped)
}
