use super::network_stub::Stub;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
pub(super) fn trials() -> Vec<Trial> {
    // The XML peer checks the exact native assertion request and refuses extra requests.
    vec![transcript_trial(
        crate::runner::required_contract::CLINVAR_ASSERTION.0,
        || assertions_wire().map_err(Into::into),
    )]
}
fn assertions_wire() -> Result<(), String> {
    let stub = Stub::for_source("eutils.ncbi.nlm.nih.gov", "application/xml", clinvar_body)?;
    let environment = super::variant::source_environment(&stub)?;
    let args = [
        "variant",
        "assertions",
        "clinvar:128042",
        "--source",
        "clinvar",
        "--assertion",
        "SCV000611258",
        "--version",
        "1",
    ]
    .iter()
    .map(|s| (*s).into())
    .collect::<Vec<_>>();
    let output = environment
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    if value.pointer("/result/assertions/0/statement/attribution/record")
        != Some(&json!("SCV000611258"))
        || value.pointer("/result/assertions/0/statement/classification/codings/0/code")
            != Some(&json!("pathogenic"))
        || value.pointer("/result/assertions/0/statement/strength/codings/0")
            != Some(
                &json!({"system":"clinvar_review_status","code":"criteria_provided_single_submitter"}),
            )
        || value.pointer("/result/assertions/0/statement/allele_origin/codings/0")
            != Some(&json!({"system":"clinvar_allele_origin","code":"unknown"}))
    {
        return Err(format!("native assertion lost: {value}"));
    }
    let requests = stub.finish()?;
    if requests.len() != 1
        || !requests.first().is_some_and(|(_, request)| {
            request.starts_with(
                "GET \
        /entrez/eutils/efetch.fcgi?db=clinvar&id=128042&rettype=vcv&is_variationid=true&retmode=xml&tool=biomcp HTTP/1.1",
            )
        })
    {
        return Err(format!("wrong exact ClinVar wire: {requests:?}"));
    }
    Ok(())
}

pub(super) fn clinvar_body(_: &str) -> Result<String, String> {
    // Explicit synthetic XML uses clinically reviewed public fixture facts; this is not a source capture.
    Ok("<ClinVarResult-Set><VariationArchive VariationID=\"128042\" Accession=\"VCV000128042\" \
        Version=\"175\"><ClassifiedRecord><Classifications><GermlineClassification><Description>Pathogenic</\
        Description></GermlineClassification></Classifications><ClinicalAssertionList><ClinicalAssertion><ClinVarAccession \
                    Accession=\"SCV000611258\" Version=\"1\" SubmitterName=\"Fulgent Genetics, Fulgent \
        Genetics\"/><Classification DateLastEvaluated=\"2017-05-18\"><ReviewStatus>criteria provided, \
        single \
        submitter</ReviewStatus><GermlineClassification>Pathogenic</GermlineClassification></Classification>\
        <AttributeSet><Attribute Type=\"AssertionMethod\">ACMG Guidelines, 2015</Attribute><Citation><ID \
        Source=\"PubMed\">25741868</ID></Citation></AttributeSet><ObservedInList><ObservedIn><Sample><Origin\
        >unknown</Origin></Sample></ObservedIn></ObservedInList><TraitSet><Trait><XRef DB=\"OMIM\" \
        ID=\"114480\"/></Trait><Trait><XRef DB=\"OMIM\" ID=\"176807\"/></Trait><Trait><XRef DB=\"OMIM\" \
        ID=\"259500\"/></Trait><Trait><XRef DB=\"OMIM\" \
        ID=\"609265\"/></Trait></TraitSet></ClinicalAssertion></ClinicalAssertionList></ClassifiedRecord></VariationArchive></ClinVarResult-Set>"
        .into())
}
