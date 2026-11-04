use jsonschema::{Retrieve, Uri, Validator};
use serde_json::Value;
use std::{collections::BTreeMap, error::Error, sync::Arc};

pub(super) const ARTIFACTS: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/schemas.rs"));
// Only a resolution anchor. The custom retriever never performs external I/O.
const BASE: &str = "https://schema.invalid/";
const DIAGNOSTIC_CHARS: usize = 160;
pub(super) struct Registry {
    artifacts: BTreeMap<String, Value>,
    validators: BTreeMap<String, Validator>,
}

impl Registry {
    pub(super) fn embedded() -> Result<Arc<Self>, String> {
        #[expect(
            clippy::disallowed_types,
            reason = "Compile the immutable embedded schemas once and share the same fallible result across trials"
        )]
        static EMBEDDED: std::sync::OnceLock<Result<Arc<Registry>, String>> =
            std::sync::OnceLock::new();
        EMBEDDED
            .get_or_init(|| {
                let artifacts = ARTIFACTS
                    .iter()
                    .map(|(id, text)| {
                        serde_json::from_str(text)
                            .map(|value| ((*id).to_owned(), value))
                            .map_err(|_| {
                                format!("schema validation: invalid artifact {}", bounded(id))
                            })
                    })
                    .collect::<Result<_, _>>()?;
                Self::new(artifacts).map(Arc::new)
            })
            .clone()
    }

    pub(super) fn new(artifacts: BTreeMap<String, Value>) -> Result<Self, String> {
        let retriever = Memory(Arc::new(
            artifacts
                .iter()
                .map(|(id, value)| (format!("{BASE}{id}"), value.clone()))
                .collect(),
        ));
        let options = jsonschema::options()
            .with_base_uri(BASE)
            .with_retriever(retriever);
        let validators = artifacts
            .iter()
            .map(|(id, value)| {
                if value.get("$id").and_then(Value::as_str) != Some(id) {
                    return Err(format!(
                        "schema validation: artifact identity mismatch {}",
                        bounded(id)
                    ));
                }
                options
                    .build(value)
                    .map(|validator| (id.clone(), validator))
                    .map_err(|_| {
                        format!("schema validation: cannot compile artifact {}", bounded(id))
                    })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            artifacts,
            validators,
        })
    }

    pub(super) fn validate(
        &self,
        value: &Value,
        args: &[String],
        success: bool,
    ) -> Result<(), String> {
        if let [command, id] = args
            && success
            && command == "schema"
            && let Some(artifact) = self.artifacts.get(id)
        {
            return if value == artifact {
                Ok(())
            } else {
                Err(format!(
                    "schema validation: raw artifact differs from requested {}",
                    bounded(id)
                ))
            };
        }
        let id = value
            .get("schema")
            .and_then(Value::as_str)
            .ok_or("schema validation: result requires a string schema identity")?;
        reject_empty_lists(value, "", id)?;
        reject_nulls(value, "", id)?;
        let validator = self
            .validators
            .get(id)
            .ok_or_else(|| format!("schema validation: unknown identity {}", bounded(id)))?;
        if let Some(error) = validator.iter_errors(value).next() {
            return Err(format!(
                "schema validation: {} at {} violates {} ({})",
                bounded(id),
                bounded(&error.instance_path().to_string()),
                bounded(&error.schema_path().to_string()),
                error.kind().keyword()
            ));
        }
        Ok(())
    }
}

fn reject_empty_lists(value: &Value, path: &str, schema: &str) -> Result<(), String> {
    match value {
        Value::Array(items) => {
            if items.is_empty() && !gnomad_filters(path, schema) {
                return Err(format!("product reply contains an empty list at {path}"));
            }
            for (index, item) in items.iter().enumerate() {
                reject_empty_lists(item, &format!("{path}/{index}"), schema)?;
            }
        }
        Value::Object(fields) => {
            for (name, item) in fields {
                let name = name.replace('~', "~0").replace('/', "~1");
                reject_empty_lists(item, &format!("{path}/{name}"), schema)?;
            }
        }
        _ => {}
    }
    Ok(())
}

// GnomadCounts.filters preserves supplied [] separately from an absent source field.
// See gnomad/variant/raw.rs and response/variant_population.rs; other empty lists still fail.
fn gnomad_filters(path: &str, schema: &str) -> bool {
    use biomcp_types::{ResponseBody, VariantCard, VariantPage, VariantPopulationResult};
    let population = if schema == VariantPopulationResult::SCHEMA {
        if let Some(index) = path
            .strip_prefix("/result/groups/")
            .and_then(|p| p.strip_suffix("/filters"))
        {
            return index.parse::<usize>().is_ok();
        }
        path.strip_prefix("/result/population/data/")
    } else {
        let candidates = if schema == VariantCard::SCHEMA {
            path.strip_prefix("/result/candidates/")
        } else if schema == VariantPage::SCHEMA {
            path.strip_prefix("/result/items/")
        } else {
            None
        };
        candidates
            .and_then(|p| p.split_once('/'))
            .filter(|(index, _)| index.parse::<usize>().is_ok())
            .and_then(|(_, p)| p.strip_prefix("sections/population/data/"))
    };
    matches!(
        population,
        Some("exome/filters" | "genome/filters" | "joint/data/filters")
    )
}

// DiseaseAssociation explicitly preserves a publisher null separately from a missing aggregator.
// Only its association-row position in the four public response types admits that value.
fn explicit_null(path: &str, schema: &str) -> bool {
    use biomcp_types::{
        DiseaseGenes, DiseasePhenotypes, PhenotypeDiseases, PhenotypeGenes, ResponseBody,
    };
    let association = [
        DiseaseGenes::SCHEMA,
        DiseasePhenotypes::SCHEMA,
        PhenotypeDiseases::SCHEMA,
        PhenotypeGenes::SCHEMA,
    ]
    .contains(&schema);
    association
        && path
            .strip_prefix("/result/items/")
            .and_then(|rest| rest.strip_suffix("/association/aggregator_knowledge_source"))
            .is_some_and(|index| index.parse::<usize>().is_ok())
}

fn reject_nulls(value: &Value, path: &str, schema: &str) -> Result<(), String> {
    match value {
        Value::Null if !explicit_null(path, schema) => {
            return Err(format!("product reply contains null at {path}"));
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                reject_nulls(item, &format!("{path}/{index}"), schema)?;
            }
        }
        Value::Object(fields) => {
            for (name, item) in fields {
                let name = name.replace('~', "~0").replace('/', "~1");
                reject_nulls(item, &format!("{path}/{name}"), schema)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Clone)]
struct Memory(Arc<BTreeMap<String, Value>>);

impl Retrieve for Memory {
    fn retrieve(&self, uri: &Uri<String>) -> Result<Value, Box<dyn Error + Send + Sync>> {
        self.0
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| "unregistered schema reference; external retrieval disabled".into())
    }
}

fn bounded(text: &str) -> String {
    let mut chars = text.chars();
    let prefix: String = chars.by_ref().take(DIAGNOSTIC_CHARS).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}
