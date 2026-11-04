//! Command inputs accept month bounds; retained requests always contain full days.
/// Search source-native adverse event reports with inclusive month or day receipt bounds.
#[derive(Debug)]
pub struct AdverseEventSearchCommand;
impl schemars::JsonSchema for AdverseEventSearchCommand {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "AdverseEventSearchCommand".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut schema = super::AdverseEventSearch::json_schema(generator);
        if let Some(fields) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
            for field in ["received_from", "received_to"] {
                if let Some(date) = fields.get_mut(field).and_then(|v| v.as_object_mut()) {
                    date.remove("format");
                }
            }
        }
        schema.insert(
            "x-reply-query-titles".into(),
            vec![<super::AdverseEventSearch as schemars::JsonSchema>::schema_name().into_owned()]
                .into(),
        );
        schema
    }
}
