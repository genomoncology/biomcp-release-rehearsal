//! Authored synthetic native shapes; these rows contain no publisher content.
use serde_json::{Value, json};
pub(super) fn clinical(id: u64, gene: &str) -> Value {
    json!({
        "id":id,"accessionId":format!("PA{}",100000+id),
        "levelOfEvidence":{"id":1,"resource":"synthetic","term":"Synthetic level","termId":"S1"},
        "location":{"id":id,"copyNumber":"n/a","diplotypes":[{"id":123}],
            "displayName":"Synthetic unchanged location","genes":[{"id":"PA200000","symbol":gene}],
            "gpPosition":-1,"haplotypes":[],"linkedObjects":[],"refSeqPosition":-1,"tagGene":false,"type":"Synthetic"},
        "name":"Synthetic clinical annotation","relatedChemicals":[],"relatedChemicalsLogic":"OR",
        "relatedGuidelines":[],"relatedLabels":[],"types":["Synthetic category"]
    })
}
pub(super) fn detail(id: u64) -> Result<Value, String> {
    let mut row = clinical(id, "DPYD");
    let fields = row.as_object_mut().ok_or("synthetic annotation object")?;
    for (key, value) in [
        ("allelePhenotypes", json!([])),
        ("conflictingVariantAnnotationIds", json!([])),
        (
            "history",
            json!([{"id":1,"date":"2020-01-01","type":"Synthetic","version":1}]),
        ),
        ("objCls", json!("SummaryAnnotation")),
        ("overrideLevel", json!(false)),
        ("pediatric", json!(false)),
        ("relatedAlleles", json!([])),
        ("relatedDiseases", json!([])),
        ("score", json!(0.0)),
        ("scoreDetails", json!("Synthetic context")),
    ] {
        fields.insert(key.into(), value);
    }
    Ok(row)
}
pub(super) fn references(class: &str) -> Value {
    json!([
        {"objCls":class,"id":"PA100001","name":"Synthetic reference one","crossReferences":[]},
        {"objCls":class,"id":"PA100002","name":"Synthetic reference two","crossReferences":[]}
    ])
}
