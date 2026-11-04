//! Recursive contract edges retain their compatibility constraints.
use super::compatibility::check;
use serde_json::json;

#[test]
fn compatibility_checks_recursive_edges_and_constraints() {
    let recursive = |name: &str| {
        json!({"$id":"biodata.example/1", "$ref":format!("#/$defs/{name}"),
        "$defs":{name:{"type":"object","properties":{
            "next":{"$ref":format!("#/$defs/{name}")}, "value":{"type":"string","minLength":1}}}}})
    };
    let old = recursive("Node");
    assert!(check(&old, &old).is_ok());
    assert!(check(&old, &recursive("Renamed")).is_ok());
    let mut root_nullable = old.clone();
    root_nullable.as_object_mut().unwrap().remove("$ref");
    root_nullable["anyOf"] = json!([{"$ref":"#/$defs/Node"},{"type":"null"}]);
    for base in [&old, &root_nullable] {
        for reverse in [false, true] {
            let mut nullable = base.clone();
            let mut branches = vec![json!({"$ref":"#/$defs/Node"}), json!({"type":"null"})];
            branches.rotate_left(usize::from(reverse));
            nullable["$defs"]["Node"]["properties"]["next"] = json!({"anyOf":branches});
            assert!(check(&nullable, &nullable).is_ok());
            assert!(check(&nullable, base).is_err());
        }
    }
    let mut mutual = old.clone();
    mutual["$defs"]["Node"]["properties"]["next"]["$ref"] = json!("#/$defs/Other");
    mutual["$defs"]["Other"] = old["$defs"]["Node"].clone();
    assert!(check(&old, &mutual).is_ok());
    assert!(check(&mutual, &old).is_ok());
    for mutation in ["narrow", "missing", "redirect", "unresolved", "external"] {
        let mut new = mutual.clone();
        let properties = &mut new["$defs"]["Other"]["properties"];
        match mutation {
            "narrow" => properties["value"]["minLength"] = json!(2),
            "missing" => {
                properties.as_object_mut().unwrap().remove("next");
            }
            "redirect" => {
                properties["next"]["$ref"] = json!("#/$defs/Leaf");
                new["$defs"]["Leaf"] = json!({"type":"object"});
            }
            bad => {
                properties["next"]["$ref"] = json!(if bad == "external" {
                    "other.json#/$defs/Node"
                } else {
                    "#/$defs/Missing"
                })
            }
        }
        assert!(check(&old, &new).is_err(), "{mutation}");
        if ["unresolved", "external"].contains(&mutation) {
            new["$id"] = json!("biodata.example/2");
            assert!(check(&old, &new).is_err());
        }
    }
    for defs in [
        json!({"A":{"$ref":"#/$defs/A"}}),
        json!({"A":{"$ref":"#/$defs/B"},"B":{"$ref":"#/$defs/A"}}),
    ] {
        let mut invalid = json!({"$id":"biodata.example/2","$ref":"#/$defs/A","$defs":defs});
        assert!(check(&old, &invalid).is_err());
        invalid["$id"] = json!("biodata.example/1");
        assert!(check(&invalid, &invalid).is_err());
    }
}
