//! Offline test-tool bridge. Each input line contains a batch of source values.
use biodata::{
    Accession, CaId, ClinVarAlleleId, ClinVarVariationId, Concept, Curie, Doi, DrugId, Expression,
    GeneSymbol, HgvsText, MondoId, NcbiGeneId, NcitId, NctId, Pmid, ProfileExpression, RcvId, RsId,
    ScvId, SourceText, Text, VariantId, VariantRef, VcvId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
struct Input {
    kind: String,
    value: Value,
}

#[derive(Debug, Serialize)]
struct Output {
    kind: String,
    value: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    parsed: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

fn checked<T: serde::de::DeserializeOwned + Serialize>(value: &Value) -> Result<Value, String> {
    let parsed: T = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    serde_json::to_value(parsed).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct ProfileInput {
    record: Text,
    label: Text,
    members: Box<[VariantRef]>,
    raw_name: Text,
}

fn profile(value: &Value) -> Result<Value, String> {
    let input: ProfileInput = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    let expression = ProfileExpression::civic(input.raw_name).map_err(|e| e.to_string())?;
    let profile =
        VariantRef::profile_expression(input.record, input.label, input.members, expression)
            .map_err(|e| e.to_string())?;
    serde_json::to_value(profile).map_err(|e| e.to_string())
}

fn parse(input: Input) -> Output {
    let value = &input.value;
    let result = match input.kind.as_str() {
        "variant" => checked::<VariantId>(value),
        "profile" => profile(value),
        "disease" | "drug" => checked::<Concept>(value),
        "drug_id" => checked::<DrugId>(value),
        "hgvs" => checked::<HgvsText>(value),
        "expression" => checked::<Expression>(value),
        "gene" => checked::<GeneSymbol>(value),
        "gene_id" => checked::<NcbiGeneId>(value),
        "citation" => checked::<Pmid>(value),
        "doi" => checked::<Doi>(value),
        "identifier" => checked::<Curie>(value),
        "native_identifier" => checked::<Text>(value),
        "accession" => checked::<Accession>(value),
        "clinvar_variation" => checked::<ClinVarVariationId>(value),
        "clinvar_allele" => checked::<ClinVarAlleleId>(value),
        "vcv" => checked::<VcvId>(value),
        "rcv" => checked::<RcvId>(value),
        "scv" => checked::<ScvId>(value),
        "rs" => checked::<RsId>(value),
        "ca" => checked::<CaId>(value),
        "mondo" => checked::<MondoId>(value),
        "ncit" => checked::<NcitId>(value),
        "nct" => checked::<NctId>(value),
        "text" => checked::<Text>(value),
        "source_text" => checked::<SourceText>(value),
        _ => Err("unsupported_value_type".into()),
    };
    let (parsed, reason) = match result {
        Ok(parsed) => (Some(parsed), None),
        Err(reason) => (None, Some(reason)),
    };
    Output {
        kind: input.kind,
        value: input.value,
        parsed,
        reason,
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "Standard streams are the offline test-tool boundary"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let inputs: Vec<Input> = serde_json::from_str(&line?)?;
        let results: Vec<Output> = inputs.into_iter().map(parse).collect();
        serde_json::to_writer(&mut output, &results)?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
