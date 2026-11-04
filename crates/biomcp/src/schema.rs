//! Schema discovery generated from checked-in artifact identities.
const SCHEMAS: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/schemas.rs"));

pub(crate) fn find(name: &str) -> Option<&'static str> {
    SCHEMAS
        .iter()
        .find_map(|(id, text)| (*id == name).then_some(*text))
}

pub(crate) fn versions() -> Vec<&'static str> {
    SCHEMAS.iter().map(|(id, _)| *id).collect()
}

const CATALOG: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/schema-catalog.rs"));
const REPLIES: &[(&str, &[&str])] = include!(concat!(env!("OUT_DIR"), "/schema-replies.rs"));

pub(crate) fn catalog() -> biomcp_types::SchemaCatalog {
    biomcp_types::SchemaCatalog {
        schemas: CATALOG
            .iter()
            .map(|(id, description)| biomcp_types::SchemaEntry {
                id: (*id).into(),
                description: (*description).into(),
            })
            .collect(),
    }
}
pub(crate) fn replies(query: &str, verb: &str, entity: &str) -> Vec<String> {
    REPLIES
        .iter()
        .find(|(id, _)| *id == query)
        .into_iter()
        .flat_map(|(_, ids)| *ids)
        .filter(|id| {
            verb != "protein"
                || id
                    .split('/')
                    .next()
                    .is_some_and(|name| name == format!("biomcp.protein-{entity}"))
        })
        .map(|id| (*id).into())
        .collect()
}

pub(crate) fn local_replies(command: &[String]) -> Vec<String> {
    use biomcp_types::{
        Health, Help, KeyHelp, RegisteredKeys, ResponseBody, SchemaCatalog, StoreClean, SyncResult,
        WikiPathwaysSync,
    };
    let Some(verb) = command.first().map(String::as_str) else {
        return Vec::new();
    };
    if verb == "sync" && command.len() == 1 {
        return vec![SyncResult::SCHEMA.into(), WikiPathwaysSync::SCHEMA.into()];
    }
    let id = match verb {
        "schema" => SchemaCatalog::SCHEMA,
        "help" if command.get(1).is_some_and(|word| word == "key") => KeyHelp::SCHEMA,
        "help" => Help::SCHEMA,
        "key" => RegisteredKeys::SCHEMA,
        "health" => Health::SCHEMA,
        "store" => StoreClean::SCHEMA,
        "sync" if command.get(1).is_some_and(|word| word == "wikipathways") => {
            WikiPathwaysSync::SCHEMA
        }
        "sync" => SyncResult::SCHEMA,
        _ => return Vec::new(),
    };
    vec![id.into()]
}
