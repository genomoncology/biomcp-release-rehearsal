//! Public adapter storage contracts keep previously saved source facts readable.
use biomcp_adapters::complex_portal::Complex;
use serde_json::json;

#[test]
fn complex_without_a_saved_problem_list_preserves_its_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let legacy = json!({"header":{"id":"CPX-715","taxon":9606,"participant_total":0,
        "matched_participant":"P38398","url":"https://www.ebi.ac.uk/complexportal/complex/CPX-715"},"members":[]});
    let saved: Complex = serde_json::from_value(legacy.clone())?;
    assert_eq!(saved.header.id.as_str(), "CPX-715");
    assert!(saved.problems.is_none());
    let mut encoded = serde_json::to_value(saved)?;
    assert_eq!(
        encoded
            .as_object_mut()
            .ok_or("complex snapshot object")?
            .remove("refusals"),
        Some(json!(false))
    );
    assert_eq!(encoded, legacy);
    Ok(())
}
