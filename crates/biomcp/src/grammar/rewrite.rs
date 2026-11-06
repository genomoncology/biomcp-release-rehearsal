//! Translate checked query fields at the public command boundary.
#![deny(clippy::wildcard_enum_match_arm)]
use super::decode::decode_query;
use biomcp_types::{QueryInvalid, Request, Rewrite};
use serde_json::Value;

pub(super) fn request(
    via: Rewrite,
    verb: &str,
    entity: &str,
    mut value: Value,
) -> Result<Request, QueryInvalid> {
    match via {
        Rewrite::Direct | Rewrite::GetPhenotype => direct(verb, entity, value),
        Rewrite::GetPgx => {
            if value.get("annotation_id").is_some() {
                if let Some(fields) = value.as_object_mut()
                    && let Some(id) = fields.remove("annotation_id")
                {
                    fields.insert("id".into(), id);
                }
                decode_query(value)
                    .map(Request::GetPgxAnnotation)
                    .map_err(|cause| cause.rename("id", "annotation_id"))
            } else {
                decode_query(value).map(Request::GetPgx)
            }
        }
        Rewrite::GetDrug => {
            if value.get("source").and_then(Value::as_str) == Some("who-pq") {
                decode_query(value).map(Request::GetWhoMedicine)
            } else {
                decode_query(value).map(Request::GetDrug)
            }
        }
        Rewrite::GetGwas => {
            let fields = value.as_object_mut().ok_or_else(QueryInvalid::default)?;
            let id = fields
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(QueryInvalid::default)?;
            let kind = if id.starts_with("association:") {
                "association"
            } else {
                "study"
            };
            fields.insert("kind".into(), kind.into());
            decode_query::<biomcp_types::GwasGet>(value).map(Request::from)
        }
        Rewrite::SearchGwas => {
            let fields = value.as_object_mut().ok_or_else(QueryInvalid::default)?;
            let positional = fields.contains_key("query");
            if let Some(query) = fields.remove("query")
                && fields.insert("gene".into(), query).is_some()
            {
                return Err(QueryInvalid::new(&["gene", "query"])
                    .with_message(biomcp_types::ErrorMessage::GwasSearchArguments));
            }
            decode_query(value)
                .map(Request::SearchGwas)
                .map_err(|cause| {
                    if positional {
                        cause.rename("gene", "query")
                    } else {
                        cause
                    }
                })
        }
        Rewrite::SearchCellLine | Rewrite::SearchAuthor => {
            if via == Rewrite::SearchAuthor {
                decode_query(value).map(Request::SearchAuthor)
            } else {
                decode_query(value).map(Request::SearchCellLine)
            }
        }
    }
}

pub(crate) fn query_value(request: &Request) -> Result<Value, serde_json::Error> {
    let mut value = checked_value(request)?;
    match request.rewrite() {
        Rewrite::Direct
        | Rewrite::GetDrug
        | Rewrite::GetGwas
        | Rewrite::SearchGwas
        | Rewrite::SearchCellLine
        | Rewrite::SearchAuthor => {}
        Rewrite::GetPgx => {
            if let Request::GetPgx(query) = request
                && query.section_limit() == query.default_section_limit()
                && let Some(fields) = value.as_object_mut()
            {
                fields.remove("section_limit");
            }
            if let Request::GetPgxAnnotation(_) = request
                && let Some(fields) = value.as_object_mut()
                && let Some(id) = fields.remove("id")
            {
                fields.insert("annotation_id".into(), id);
            }
        }
        Rewrite::GetPhenotype => {
            if let Request::GetPhenotype(query) = request
                && let Some(fields) = value.as_object_mut()
            {
                if query.source() == query.default_source() {
                    fields.remove("source");
                }
                if query.section_limit() == query.default_section_limit() {
                    fields.remove("section_limit");
                }
            }
        }
    }
    Ok(value)
}

macro_rules! boundary {
    (rewrites { $($rewrite:ident),+ } $( $(#[$doc:meta])* $variant:ident($query:ident) =>
        ($verb:literal, $entity:literal, $schema_type:ident, $stem:literal, $schema:literal,
         $grammar_type:ident, $grammar_stem:literal, $grammar_schema:literal, $spelling_schema:literal, $order:literal, $via:ident); )+) => {
        fn checked_value(request: &Request) -> Result<Value, serde_json::Error> {
            match request { $(Request::$variant(query) => serde_json::to_value(query),)+ }
        }
        fn direct(verb: &str, entity: &str, value: Value) -> Result<Request, QueryInvalid> {
            match (verb, entity) {
                $(($verb, $entity) if matches!(Rewrite::$via, Rewrite::Direct | Rewrite::GetPhenotype) =>
                    boundary!(@decode $via, $variant, value),)+
                _ => Err(QueryInvalid::default()),
            }
        }
    };
    (@decode Direct, ArticleText, $value:expr) => {
        decode_query::<biomcp_types::ArticleText>($value).map(biomcp_types::ArticleText::into_request)
    };
    (@decode Direct, $variant:ident, $value:expr) => { decode_query($value).map(Request::$variant) };
    (@decode GetPhenotype, $variant:ident, $value:expr) => { decode_query($value).map(Request::$variant) };
    (@decode $rewrite:ident, $variant:ident, $value:expr) => { Err(QueryInvalid::default()) };
}
biomcp_types::command_types!(boundary);
