//! Schema-only union of the two checked PGx get selectors.
/// CLI grammar preserves distinct gene and exact annotation request contracts.
#[cfg(feature = "schema")]
#[derive(Debug)]
pub struct PgxGetCommand;
#[cfg(feature = "schema")]
impl schemars::JsonSchema for PgxGetCommand {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "PgxGetCommand".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut schema = super::PgxGet::json_schema(generator);
        let annotation = super::PgxAnnotationGet::json_schema(generator);
        if let Some(properties) = schema.get_mut("properties").and_then(|v| v.as_object_mut())
            && let Some(id) = annotation.pointer("/properties/id")
        {
            let mut id = id.clone();
            if let Some(fields) = id.as_object_mut() {
                fields.insert(
                    "x-required-component".into(),
                    crate::Source::ClinPgx.to_string().into(),
                );
            }
            properties.insert("annotation_id".into(), id);
        }
        if let Some(required) = schema.get_mut("required").and_then(|v| v.as_array_mut()) {
            required.retain(|value| value.as_str() != Some("gene"));
        }
        schema.insert(
            "x-reply-query-titles".into(),
            vec![<super::PgxAnnotationGet as schemars::JsonSchema>::schema_name().into_owned()]
                .into(),
        );
        schema
    }
}
