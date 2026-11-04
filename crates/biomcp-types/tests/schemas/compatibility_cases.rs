use super::{
    compatibility::{check, check_reviewed},
    policy::{ContractKind, Reviews},
};
use serde_json::{Value, json};

fn contract(properties: Value) -> Value {
    json!({"$id":"biodata.example/1", "type":"object", "properties":properties,
        "required":["name"], "additionalProperties":false})
}

#[test]
fn compatibility_preserves_paths_types_and_required_fields() {
    let old = contract(json!({"name":{"$ref":"#/$defs/Text"}}));
    let mut old = old;
    old["$defs"] = json!({"Text":{"type":"string"}});
    let cases = [
        (
            "optional addition",
            contract(json!({"name":{"type":"string"},"extra":{"type":"string"}})),
            true,
        ),
        ("removal", contract(json!({})), false),
        (
            "rename",
            contract(json!({"label":{"type":"string"}})),
            false,
        ),
        (
            "retyping",
            contract(json!({"name":{"type":"object"}})),
            false,
        ),
        (
            "unresolved reference",
            contract(json!({"name":{"$ref":"#/$defs/Missing"}})),
            false,
        ),
        (
            "unsupported form",
            contract(json!({"name":{"oneOf":[{"type":"string"}]}})),
            false,
        ),
        (
            "external reference",
            contract(json!({"name":{"$ref":"https://example.org/text"}})),
            false,
        ),
    ];
    for (name, new, expected) in cases {
        assert_eq!(check(&old, &new).is_ok(), expected, "{name}");
    }
    let mut new = contract(json!({"name":{"type":"string"},"extra":{"type":"string"}}));
    new["required"] = json!(["name", "extra"]);
    assert!(
        check(&old, &new).is_err(),
        "required addition breaks old inputs"
    );
    // This fixture deliberately documents a breaking rename under a new major.
    let mut new = contract(json!({"label":{"type":"object"}}));
    new["$id"] = json!("biodata.example/2");
    assert!(check(&old, &new).is_ok());
    assert!(check(&new, &old).is_err(), "major cannot decrease");
}

#[test]
fn compatibility_checks_enums_and_string_constraints() {
    let old = json!({"$id":"biodata.example/1", "type":"string", "enum":["first","second"]});
    for (values, compatible) in [
        (json!(["first", "second", "third"]), true),
        (json!(["first"]), false),
    ] {
        let mut new = old.clone();
        new["enum"] = values;
        assert_eq!(check(&old, &new).is_ok(), compatible);
    }
    let old = json!({"$id":"biodata.example/1", "type":"string", "minLength":1});
    let mut new = old.clone();
    new["minLength"] = json!(2);
    assert!(check(&old, &new).is_err());
}

#[test]
fn compatibility_refuses_assertion_siblings_on_composed_schemas() {
    let old = json!({"$id":"biodata.example/1", "type":"string", "enum":["first","second"]});
    // These assertions intersect; overwriting the referenced enum would hide
    // the removal of "second" from the accepted values.
    let new = json!({"$id":"biodata.example/1", "$ref":"#/$defs/Value",
        "$defs":{"Value":{"type":"string","enum":["first"]}},
        "enum":["first","second"]});
    assert!(check(&old, &new).is_err());

    let old = json!({"$id":"biodata.example/1", "type":["string","null"]});
    let new = json!({"$id":"biodata.example/1", "anyOf":[{"type":"string"},{"type":"null"}], "type":"string"});
    assert!(check(&old, &new).is_err(), "a sibling can exclude null");
}

#[test]
fn compatibility_preserves_patterns_and_rejects_tighter_integer_bounds() {
    let old = json!({"$id":"biodata.example/1", "type":"string", "pattern":"^a$"});
    for pattern in [json!("^b$"), json!(null)] {
        let mut new = old.clone();
        if pattern.is_null() {
            new.as_object_mut().unwrap().remove("pattern");
        } else {
            new["pattern"] = pattern;
        }
        assert!(check(&old, &new).is_err());
    }
    let old = json!({"$id":"biodata.example/1", "type":"integer", "minimum":1,"maximum":100});
    for (minimum, maximum, expected) in [
        (0, 101, true),
        (1, 100, true),
        (2, 100, false),
        (1, 99, false),
    ] {
        let mut new = old.clone();
        new["minimum"] = json!(minimum);
        new["maximum"] = json!(maximum);
        assert_eq!(check(&old, &new).is_ok(), expected);
    }
    for unsupported in [
        json!({"maximum":1.5}),
        json!({"format":"int32"}),
        json!({"exclusiveMinimum":0}),
    ] {
        let mut new = old.clone();
        new.as_object_mut()
            .unwrap()
            .extend(unsupported.as_object().unwrap().clone());
        assert!(check(&old, &new).is_err());
    }
}

#[test]
fn compatibility_tracks_array_items_and_each_tagged_variant() {
    let old = json!({"$id":"biodata.example/1","type":"array","items":{"type":"string"}});
    let mut new = old.clone();
    new["items"]["type"] = json!("integer");
    assert!(check(&old, &new).is_err());
    fn branch(tag: &str, fields: Value) -> Value {
        let mut properties = fields.as_object().unwrap().clone();
        properties.insert("resolution".into(), json!({"type":"string","const":tag}));
        json!({"type":"object","properties":properties,"required":["resolution"],"additionalProperties":false})
    }
    let old = json!({"$id":"biodata.example/1","oneOf":[branch("a", json!({"name":{"type":"string"}})), branch("b",json!({"name":{"type":"string"}}))]});
    let reordered = json!({"$id":"biodata.example/1","oneOf":[branch("b",json!({"name":{"type":"string"}})),branch("a",json!({"name":{"type":"string"}}))]});
    assert!(check(&old, &reordered).is_ok());
    let removed = json!({"$id":"biodata.example/1","oneOf":[branch("a",json!({})),branch("b",json!({"name":{"type":"string"}}))]});
    assert!(check(&old, &removed).is_err());
    let retyped = json!({"$id":"biodata.example/1","oneOf":[branch("a",json!({"name":{"type":"integer"}})),branch("b",json!({"name":{"type":"string"}}))]});
    assert!(check(&old, &retyped).is_err());
}

#[test]
fn compatibility_preserves_consumed_boolean_and_array_bounds() {
    let old = json!({"$id":"biodata.example/1","type":"array","minItems":1,"maxItems":20,"items":{"type":"boolean"}});
    assert!(check(&old, &old).is_ok());
    for (key, value) in [("minItems", json!(2)), ("maxItems", json!(19))] {
        let mut new = old.clone();
        new[key] = value;
        assert!(check(&old, &new).is_err());
    }
    let mut relaxed = old.clone();
    relaxed["minItems"] = json!(0);
    relaxed["maxItems"] = json!(21);
    assert!(check(&old, &relaxed).is_ok());
}

#[test]
fn selector_unions_accept_only_required_lists_and_detect_narrowing() {
    let base = json!(
        {"$id":"biodata.example/1","type":"object","properties":{"query":{"type":"string"},"band":{"type":"string"}},
        "anyOf":[{"required":["query"]},{"required":["band"]}]}
    );
    assert!(check(&base, &base).is_ok());
    let mut narrow = base.clone();
    narrow["anyOf"] = json!([{"required":["query"]}]);
    assert!(check(&base, &narrow).is_err());
    assert!(check(&narrow, &base).is_ok());
    // Required query inputs already satisfy an added query-or-filter selector.
    let mut required = base.clone();
    required.as_object_mut().unwrap().remove("anyOf");
    required["required"] = json!(["query"]);
    assert!(check(&required, &base).is_ok());
    let mut unrelated = base.clone();
    unrelated["anyOf"] = json!([{"required":["band"]}]);
    assert!(check(&required, &unrelated).is_err());
    assert!(check_reviewed(&required, &base, ContractKind::Reply, &Reviews::default()).is_err());
    let referenced = |contract: &Value| json!({"$id":"biodata.example/1", "$defs":{"Search":contract}, "$ref":"#/$defs/Search"});
    assert!(check(&referenced(&base), &referenced(&narrow)).is_err());
    for branches in [
        json!([]),
        json!([{"required":[]}]),
        json!([{"required":["missing"]}]),
        json!([{"required":["query"], "type":"object"}]),
        json!([{"properties":{}}]),
    ] {
        let mut invalid = base.clone();
        invalid["anyOf"] = branches;
        assert!(check(&invalid, &invalid).is_err());
    }
}

#[test]
fn compatibility_checks_consumed_decimal_bounds_and_section_annotation() {
    let old = json!({"$id":"biodata.example/1","type":"number","format":"double","minimum":-2.5,"maximum":3.25});
    assert!(check(&old, &old).is_ok());
    for (minimum, maximum, expected) in [(-3.0, 4.0, true), (-2.0, 3.25, false), (-2.5, 3.0, false)]
    {
        let mut new = old.clone();
        new["minimum"] = json!(minimum);
        new["maximum"] = json!(maximum);
        assert_eq!(check(&old, &new).is_ok(), expected);
    }
    for name in ["x-trailing-section", "x-comma-separated", "x-identifier"] {
        let mut annotation =
            json!({"$id":"biodata.example/1","type":"array","items":{"type":"string"}});
        annotation[name] = json!(true);
        assert!(check(&annotation, &annotation).is_ok());
        annotation[name] = json!("all");
        assert!(check(&annotation, &annotation).is_err());
    }
}

#[test]
fn compatibility_preserves_markdown_rendering_contract() {
    let plain = json!({"$id":"biodata.example/1", "type":"string"});
    let mut markdown = plain.clone();
    markdown["format"] = json!("markdown");
    assert!(check(&plain, &markdown).is_ok());
    assert!(check(&markdown, &plain).is_err());
    assert!(check(&markdown, &markdown).is_ok());
}

#[test]
fn cli_flag_annotations_are_metadata_with_checked_shape() {
    let old = contract(json!({"name":{"type":"string"}}));
    let new = contract(
        json!({"name":{"type":"string", "x-cli-flags":{"include-retracted":"include", "exclude-retracted":"exclude"}}}),
    );
    assert!(check(&old, &new).is_ok());
    for flags in [
        json!(true),
        json!({}),
        json!({"--bad":"include"}),
        json!({"include":false}),
        json!({"include":""}),
    ] {
        let invalid = contract(json!({"name":{"type":"string", "x-cli-flags":flags}}));
        assert!(check(&old, &invalid).is_err());
    }
}

#[test]
fn compatibility_checks_unique_array_items_without_waiving_versions() {
    let old = contract(json!({"rows":{"type":"array","items":{"type":"string"}}}));
    let mut unique = old.clone();
    unique["properties"]["rows"]["uniqueItems"] = json!(true);
    assert!(check(&unique, &unique).is_ok());
    assert!(check(&old, &unique).is_err());
    assert!(check(&unique, &old).is_ok());
    unique["$id"] = json!("biodata.example/2");
    assert!(check(&old, &unique).is_ok());
    unique["properties"]["rows"]["uniqueItems"] = json!("true");
    assert!(check(&unique, &unique).is_err());
}

#[test]
fn compatibility_checks_native_gene_maps_signed_positions_and_impossible_elements() {
    let old = json!(
        {"$id":"biodata.example/1","type":"object","properties":{"genes":{"type":"object","propertyNames":{"type":"string",
        "pattern":"^[A-Z]+$"},"additionalProperties":{"type":"string"}},"position":{"type":"integer","format":"int64","minimum":-1},
        "empty":{"type":"array","items":false}}}
    );
    assert!(super::compatibility::check(&old, &old).is_ok());
    for changed in [
        json!({"type":"integer"}),
        json!({"type":"string","minLength":2}),
    ] {
        let mut new = old.clone();
        new["properties"]["genes"]["additionalProperties"] = changed;
        assert!(super::compatibility::check(&old, &new).is_err());
    }
    let mut changed = old.clone();
    changed["properties"]["position"]["minimum"] = json!(0);
    assert!(super::compatibility::check(&old, &changed).is_err());
    let mut changed = old.clone();
    changed["properties"]["genes"]["propertyNames"]["pattern"] = json!("^[A-Z]{2}$");
    assert!(super::compatibility::check(&old, &changed).is_err());
    let pattern = json!({"$id":"biodata.example/1","type":"object","patternProperties":{"^[A-Z]+$":{"type":"string"}},"additionalProperties":false});
    assert!(super::compatibility::check(&pattern, &pattern).is_ok());
    let mut changed = pattern.clone();
    changed["patternProperties"]["^[A-Z]+$"] = json!({"type":"integer"});
    assert!(super::compatibility::check(&pattern, &changed).is_err());
    let mut changed = pattern.clone();
    changed["patternProperties"]["X"] = json!({"type":"string"});
    assert!(super::compatibility::check(&pattern, &changed).is_err());
    let impossible = json!({"$id":"biodata.example/1","type":"array","items":{"not":{}}});
    assert!(super::compatibility::check(&impossible, &impossible).is_ok());
    for negation in [
        json!({"not":{"type":"string"}}),
        json!({"not":{},"type":"string"}),
    ] {
        let mut changed = impossible.clone();
        changed["items"] = negation;
        assert!(super::compatibility::check(&impossible, &changed).is_err());
    }
    let unrestricted_keys = json!({"$id":"biodata.example/1","type":"object","propertyNames":true});
    let mut no_keys = unrestricted_keys.clone();
    no_keys["propertyNames"] = json!(false);
    assert!(
        check(&unrestricted_keys, &no_keys).is_err(),
        "false propertyNames forbids every nonempty object"
    );
    let loose = json!({"$id":"biodata.example/1","type":"object","additionalProperties":true});
    for changed in [
        json!({"$id":"biodata.example/1","type":"object","additionalProperties":{"type":"string"}}),
        json!({"$id":"biodata.example/1","type":"object","propertyNames":{"type":"string","pattern":"X"}}),
    ] {
        assert!(super::compatibility::check(&loose, &changed).is_err());
    }
}

#[test]
fn compatibility_preserves_standard_string_format_annotations_and_assertions() {
    let plain = json!({"$id":"biodata.example/1", "type":"string", "pattern":"^x"});
    for format in ["date", "date-time", "uri", "uri-reference"] {
        let mut annotated = plain.clone();
        annotated["format"] = json!(format);
        // Draft 2020-12 uses the format annotation vocabulary by default.
        assert!(check(&plain, &annotated).is_ok());
        assert!(check(&annotated, &annotated).is_ok());
        assert!(check(&annotated, &plain).is_err());
        let mut narrowed = annotated.clone();
        narrowed["minLength"] = json!(2);
        assert!(check(&plain, &narrowed).is_err());
        narrowed = annotated.clone();
        narrowed["pattern"] = json!("^xx");
        assert!(check(&plain, &narrowed).is_err());
        narrowed = annotated.clone();
        narrowed["type"] = json!("integer");
        assert!(check(&narrowed, &narrowed).is_err());
        narrowed = annotated.clone();
        narrowed["format"] = json!(if format == "uri" {
            "uri-reference"
        } else {
            "uri"
        });
        assert!(check(&annotated, &narrowed).is_err());
    }
}

#[test]
fn compatibility_accepts_rfc_annotations_beside_string_references_without_assertion_siblings() {
    for old in [
        json!({"$id":"biodata.example/1", "$ref":"#/$defs/Text",
            "$defs":{"Text":{"type":"string","minLength":1}}}),
        json!({"$id":"biodata.example/1", "anyOf":[{"$ref":"#/$defs/Text"},{"type":"null"}],
            "$defs":{"Text":{"type":"string","minLength":1}}}),
    ] {
        for format in ["date", "date-time", "uri", "uri-reference"] {
            let mut new = old.clone();
            new["format"] = json!(format);
            assert!(check(&old, &new).is_ok());
            assert!(check(&new, &old).is_err());
            new["minLength"] = json!(2);
            assert!(check(&old, &new).is_err());
        }
        let mut unknown = old.clone();
        unknown["format"] = json!("unchecked");
        assert!(check(&old, &unknown).is_err());
    }
}

#[test]
fn compatibility_preserves_each_disjoint_scalar_branch() {
    let old = json!({"$id":"biodata.example/1", "anyOf":[
        {"$ref":"#/$defs/Finite"},
        {"type":"object","properties":{"kind":{"type":"string","enum":["nan","positive_infinity","negative_infinity"]}},
         "required":["kind"],"additionalProperties":false}],
        "$defs":{"Finite":{"type":"number","format":"double"}}});
    assert!(check(&old, &old).is_ok());
    let mut reordered = old.clone();
    reordered["anyOf"].as_array_mut().unwrap().reverse();
    assert!(check(&old, &reordered).is_ok());
    let mut number_narrowed = old.clone();
    number_narrowed["$defs"]["Finite"]["minimum"] = json!(0);
    assert!(check(&old, &number_narrowed).is_err());
    let mut object_narrowed = old.clone();
    object_narrowed["anyOf"][1]["properties"]["kind"]["enum"] = json!(["nan"]);
    assert!(check(&old, &object_narrowed).is_err());
    for unsupported in [
        json!([{"type":"number"},{"type":"string"}]),
        json!([{"type":"object"},{"type":"object"}]),
        json!([{"type":"number"},{"type":"object"},{"type":"null"}]),
    ] {
        let mut new = old.clone();
        new["anyOf"] = unsupported;
        assert!(check(&new, &new).is_err());
    }
}

#[test]
fn compatibility_keeps_source_date_and_timestamp_object_constraints() {
    let old = json!({"$id":"biodata.example/1", "anyOf":[
        {"type":"string","pattern":"^[0-9]{4}$"},
        {"type":"object","properties":{"value":{"type":"string","format":"date-time"}},
            "required":["value"]}]});
    assert!(check(&old, &old).is_ok());
    let mut reordered = old.clone();
    reordered["anyOf"].as_array_mut().unwrap().reverse();
    assert!(check(&old, &reordered).is_ok());
    for index in [0, 1] {
        let mut removed = old["anyOf"][index].clone();
        removed["$id"] = json!("biodata.example/1");
        assert!(check(&old, &removed).is_err(), "union branch disappeared");
    }
    let mut narrowed = old.clone();
    narrowed["anyOf"][0]["pattern"] = json!("^2024$");
    assert!(
        check(&old, &narrowed).is_err(),
        "date branch narrowed within a major"
    );
    narrowed = old.clone();
    narrowed["anyOf"][1]["properties"]["value"]["minLength"] = json!(30);
    assert!(
        check(&old, &narrowed).is_err(),
        "timestamp object branch narrowed within a major"
    );
    narrowed["$id"] = json!("biodata.example/2");
    assert!(
        check(&old, &narrowed).is_ok(),
        "valid successor was rejected"
    );
}

#[test]
fn required_reply_names_cannot_disappear_within_a_major() {
    let old = json!({"$id":"biodata.example/1","type":"object","properties":{
        "name":{"type":"string"}},"required":["name"]});
    let mut new = old.clone();
    new["required"] = json!([]);
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &Reviews::default()).is_err());
    assert!(check_reviewed(&old, &new, ContractKind::Query, &Reviews::default()).is_ok());
    assert!(check_reviewed(&new, &old, ContractKind::Reply, &Reviews::default()).is_ok());
    assert!(check_reviewed(&new, &old, ContractKind::Query, &Reviews::default()).is_err());
    new["$id"] = json!("biodata.example/2");
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &Reviews::default()).is_ok());
}

#[test]
fn pattern_widening_requires_an_exact_reviewed_transition() {
    let old = json!({"$id":"biodata.example/1","type":"string","pattern":"^a$"});
    let mut new = old.clone();
    new["pattern"] = json!("^[ab]$");
    let reviews: Reviews = serde_json::from_value(json!({"pattern_widenings":[{
        "schema":"biodata.example/1", "path":"", "from":"^a$", "to":"^[ab]$",
        "review":"Accept the additional canonical spelling"
    }]}))
    .unwrap();
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &Reviews::default()).is_err());
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &reviews).is_ok());
    new["pattern"] = json!("^b$");
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &reviews).is_err());
}

#[test]
fn successor_identity_requires_a_review_and_a_new_major() {
    let old = json!({"$id":"biodata.example/1","type":"string"});
    let mut new = json!({"$id":"biodata.renamed/2","type":"string"});
    let reviews: Reviews = serde_json::from_value(json!({"successors":[{
        "from":"biodata.example/1", "to":"biodata.renamed/2", "review":"Rename the contract"
    }]}))
    .unwrap();
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &Reviews::default()).is_err());
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &reviews).is_ok());
    new["$id"] = json!("biodata.renamed/1");
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &reviews).is_err());
}

#[test]
fn growable_enum_contracts_keep_known_values_and_accept_future_names() {
    let old = json!({"$id":"biodata.example/1", "type":"string",
        "x-extensible-enum":["first","second"]});
    let mut new = old.clone();
    new["x-extensible-enum"] = json!(["first", "second", "third"]);
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &Reviews::default()).is_ok());
    new["x-extensible-enum"] = json!(["first"]);
    assert!(check_reviewed(&old, &new, ContractKind::Reply, &Reviews::default()).is_err());
    let mut narrowed = old.clone();
    narrowed["enum"] = json!(["first", "second"]);
    assert!(check_reviewed(&old, &narrowed, ContractKind::Reply, &Reviews::default()).is_err());
    let closed = json!({"$id":"biodata.example/1", "type":"string", "enum":["first","second"]});
    assert!(check_reviewed(&old, &closed, ContractKind::Reply, &Reviews::default()).is_err());
}

#[test]
fn extensible_annotations_cannot_hide_enum_or_const_assertions_through_references() {
    for assertion in [json!({"enum":["first"]}), json!({"const":"first"})] {
        let mut value =
            json!({"$id":"biodata.example/1", "type":"string", "x-extensible-enum":["first"]});
        value
            .as_object_mut()
            .unwrap()
            .extend(assertion.as_object().unwrap().clone());
        assert!(check_reviewed(&value, &value, ContractKind::Reply, &Reviews::default()).is_err());
        let mut referenced = json!({"$id":"biodata.example/1", "$ref":"#/$defs/E", "x-extensible-enum":["first"],
            "$defs":{"E":{"type":"string"}}});
        referenced["$defs"]["E"]
            .as_object_mut()
            .unwrap()
            .extend(assertion.as_object().unwrap().clone());
        assert!(
            check_reviewed(
                &referenced,
                &referenced,
                ContractKind::Reply,
                &Reviews::default()
            )
            .is_err()
        );
    }
    let optional = json!({"$id":"biodata.example/1", "anyOf":[{"$ref":"#/$defs/E"},{"type":"null"}],
        "$defs":{"E":{"type":"string","x-extensible-enum":["first"]}}});
    assert!(
        check_reviewed(
            &optional,
            &optional,
            ContractKind::Reply,
            &Reviews::default()
        )
        .is_ok()
    );
    let optional = json!({"$id":"biodata.example/1", "anyOf":[
        {"type":"string","x-extensible-enum":["first"]},{"type":"null"}]});
    assert!(
        check_reviewed(
            &optional,
            &optional,
            ContractKind::Reply,
            &Reviews::default()
        )
        .is_ok()
    );
}

#[test]
fn compatibility_checks_string_length_caps() {
    let uncapped = json!({"$id":"biodata.example/1", "type":"string"});
    let mut capped = uncapped.clone();
    capped["maxLength"] = json!(100);
    assert!(
        check(&uncapped, &capped).is_err(),
        "a new cap narrows admission"
    );
    for (limit, expected) in [(99, false), (100, true), (101, true)] {
        let mut new = capped.clone();
        new["maxLength"] = json!(limit);
        assert_eq!(check(&capped, &new).is_ok(), expected, "maxLength {limit}");
    }
    assert!(
        check(&capped, &uncapped).is_ok(),
        "removing a cap widens admission"
    );
    capped["$id"] = json!("biodata.example/2");
    assert!(
        check(&uncapped, &capped).is_ok(),
        "a new major permits narrowing"
    );
    capped["maxLength"] = json!(-1);
    assert!(
        check(&uncapped, &capped).is_err(),
        "negative caps are malformed"
    );
}
