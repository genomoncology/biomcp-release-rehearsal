//! Reviewed exceptions bind exact schema identities and pattern transitions.
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub(super) enum ContractKind {
    Query,
    Reply,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Reviews {
    #[serde(default)]
    pattern_widenings: Vec<PatternWidening>,
    #[serde(default)]
    pattern_narrowings: Vec<PatternNarrowing>,
    #[serde(default)]
    successors: Vec<Successor>,
    #[serde(default)]
    enum_transitions: Vec<EnumTransition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PatternWidening {
    schema: String,
    path: String,
    from: Option<String>,
    to: Option<String>,
    review: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PatternNarrowing {
    from: String,
    to: String,
    review: String,
    sites: Vec<PatternSite>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PatternSite {
    schema: String,
    paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Successor {
    from: String,
    to: String,
    review: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumTransition {
    schemas: Vec<String>,
    path: String,
    from: Vec<String>,
    to: Vec<String>,
    review: String,
}

impl Reviews {
    /// Replace only an explicitly reviewed enum node in a private baseline copy.
    pub(super) fn enum_baseline(&self, old: &Value, new: &Value) -> Value {
        let mut baseline = old.clone();
        for entry in &self.enum_transitions {
            let from = serde_json::json!(entry.from);
            let to = serde_json::json!(entry.to);
            if old["$id"] == new["$id"]
                && entry.schemas.iter().any(|schema| old["$id"] == *schema)
                && !entry.review.trim().is_empty()
                && old.pointer(&entry.path) == Some(&from)
                && new.pointer(&entry.path) == Some(&to)
                && let Some(node) = baseline.pointer_mut(&entry.path)
            {
                *node = to;
            }
        }
        baseline
    }

    pub(super) fn pattern(
        &self,
        name: &str,
        major: u64,
        path: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> bool {
        self.pattern_widenings.iter().any(|entry| {
            entry.schema == format!("{name}/{major}")
                && entry.path == path
                && entry.from.as_deref() == from
                && entry.to.as_deref() == to
                && !entry.review.trim().is_empty()
        })
    }

    pub(super) fn successor(&self, old: &Value, new: &Value) -> bool {
        self.successors.iter().any(|entry| {
            old["$id"] == entry.from && new["$id"] == entry.to && !entry.review.trim().is_empty()
        })
    }

    pub(super) fn narrowing(
        &self,
        name: &str,
        major: u64,
        path: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> bool {
        self.pattern_narrowings.iter().any(|entry| {
            from == Some(entry.from.as_str())
                && to == Some(entry.to.as_str())
                && !entry.review.trim().is_empty()
                && entry.sites.iter().any(|site| {
                    site.schema == format!("{name}/{major}")
                        && site.paths.iter().any(|approved| approved == path)
                })
        })
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "Schema contract tests own the independently supplied review file and environment I/O"
)]
pub(super) fn read_reviews() -> Result<Reviews, Box<dyn std::error::Error>> {
    match std::env::var_os("BIOMCP_SCHEMA_REVIEWS") {
        Some(path) => Ok(serde_json::from_slice(&std::fs::read(path)?)?),
        None => Ok(serde_json::from_str(include_str!(
            "../../../../release/schema-reviews.json"
        ))?),
    }
}

#[test]
fn narrowing_requires_the_exact_schema_path_and_patterns() {
    use super::compatibility::check_reviewed;
    use serde_json::json;
    let from = "^[1-9][0-9]*$";
    let to = "^[1-9][0-9]{0,8}$";
    let reviews: Reviews = serde_json::from_value(json!({"pattern_narrowings":[{
        "from":from,"to":to,"review":"Reviewed integer bound",
        "sites":[{"schema":"biodata.example/1","paths":["/id"]}]
    }]}))
    .expect("valid review fixture");
    let schema = |name: &str, field: &str, pattern: &str| {
        json!({"$id":name,"type":"object","properties":{
            field:{"type":"string","pattern":pattern}
        }})
    };
    for kind in [ContractKind::Query, ContractKind::Reply] {
        for (name, field, before, after, accepted) in [
            ("biodata.example/1", "id", from, to, true),
            ("biodata.example/1", "other", from, to, false),
            ("biodata.unrelated/1", "id", from, to, false),
            ("biodata.example/2", "id", from, to, false),
            ("biodata.example/1", "id", "^[0-9]+$", to, false),
            ("biodata.example/1", "id", from, "^[1-9][0-9]{0,9}$", false),
            ("biodata.example/1", "id", to, from, false),
        ] {
            assert_eq!(
                check_reviewed(
                    &schema(name, field, before),
                    &schema(name, field, after),
                    kind,
                    &reviews
                )
                .is_ok(),
                accepted,
                "{name} {field}: {before} -> {after}"
            );
        }
    }
}

#[test]
fn narrowing_policy_rejects_unknown_fields() {
    for value in [
        serde_json::json!({"pattern_narrowngs":[]}),
        serde_json::json!({"pattern_narrowings":[{"from":"a","to":"b","review":"ok","sites":[],"typo":true}]}),
        serde_json::json!({"pattern_narrowings":[{"from":"a","to":"b","review":"ok","sites":[{"schema":"x/1","paths":[],"typo":true}]}]}),
    ] {
        assert!(serde_json::from_value::<Reviews>(value).is_err());
    }
}

#[test]
fn enum_review_binds_schema_site_and_exact_values_without_masking_constraints() {
    use super::compatibility::check_reviewed;
    use serde_json::json;
    let from = json!(["open", "attribution", "noncommercial", "restricted"]);
    let to = json!([
        "open",
        "attribution",
        "noncommercial",
        "licensed",
        "per-record"
    ]);
    let reviews: Reviews = serde_json::from_value(json!({"enum_transitions":[{
        "schemas":["biomcp.example/1"], "path":"/$defs/Use/enum",
        "from":from, "to":to, "review":"Reviewed source-use transition"
    }]}))
    .expect("valid enum review fixture");
    let schema = |values: &Value| {
        json!({"$id":"biomcp.example/1", "type":"object",
            "properties":{"use":{"$ref":"#/$defs/Use"},"other":{"type":"string","enum":["a","b"]}},
            "$defs":{"Use":{"type":"string","enum":values}}
        })
    };
    let old = schema(&from);
    let new = schema(&to);
    for kind in [ContractKind::Query, ContractKind::Reply] {
        assert!(check_reviewed(&old, &new, kind, &reviews).is_ok());
        assert!(check_reviewed(&old, &new, kind, &Reviews::default()).is_err());
        for changed in [
            {
                let mut v = new.clone();
                v["$id"] = json!("biomcp.other/1");
                v
            },
            {
                let mut v = new.clone();
                v["$defs"]["Use"]["enum"] = json!(["open", "licensed"]);
                v
            },
            {
                let mut v = new.clone();
                v["properties"]["other"]["enum"] = json!(["a"]);
                v
            },
            {
                let mut v = new.clone();
                v["$defs"]["Use"]["type"] = json!("integer");
                v
            },
            {
                let mut v = new.clone();
                v["$defs"]["Use"]["maxLength"] = json!(2);
                v
            },
        ] {
            assert!(
                check_reviewed(&old, &changed, kind, &reviews).is_err(),
                "{changed}"
            );
        }
        let wrong_site: Reviews = serde_json::from_value(json!({"enum_transitions":[{
            "schemas":["biomcp.example/1"], "path":"/properties/other/enum",
            "from":from, "to":to, "review":"Different site"
        }]}))
        .expect("valid other-site fixture");
        assert!(check_reviewed(&old, &new, kind, &wrong_site).is_err());
    }
}

#[test]
fn enum_review_rejects_unknown_fields_and_nonstring_values() {
    for value in [
        serde_json::json!({"enum_transitions":[{"schemas":[],"path":"/enum","from":[],"to":[],"review":"ok","typo":true}]}),
        serde_json::json!({"enum_transitions":[{"schemas":[],"path":"/enum","from":[1],"to":[],"review":"ok"}]}),
    ] {
        assert!(serde_json::from_value::<Reviews>(value).is_err());
    }
}
