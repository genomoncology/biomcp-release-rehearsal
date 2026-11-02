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

impl Reviews {
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
