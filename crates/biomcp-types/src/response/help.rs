use serde::{Deserialize, Serialize};
/// Requested help path, with the root represented by an empty object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HelpQuery {
    /// Command words whose help was requested.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
}
/// Generated command reference from the shipped query schemas and cases.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Help {
    /// Commands available at this help path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<HelpCommand>,
}
impl super::ResponseBody for Help {
    const SCHEMA: &'static str = "biomcp.help/6";
    type Query = HelpQuery;
}
/// One command and its generated arguments.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HelpCommand {
    /// Command words after the executable name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
    /// Query type documentation.
    pub description: String,
    /// Canonical reply contracts; empty for commands with no schema reply.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reply_schemas: Vec<String>,
    /// Positional arguments and flags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<HelpArgument>,
    /// Accepted section words and their documentation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<HelpWord>,
    /// How to disambiguate section words from inputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_rule: Option<String>,
    /// Verbatim case run steps, without replay configuration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<Vec<String>>,
}
/// One schema argument, retaining its bounds and position.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HelpArgument {
    /// Positional name or long flag.
    pub name: String,
    /// Alternative accepted flags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Value used when the argument is omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Field documentation.
    pub description: String,
    /// Schema scalar or array type.
    pub value_type: String,
    /// Whether the query requires this argument.
    pub required: bool,
    /// Zero based positional index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    /// Lower numeric or array length bound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<u64>,
    /// Upper numeric or array length bound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<u64>,
    /// Allowed scalar values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
}
/// A documented closed vocabulary word.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HelpWord {
    /// Accepted spelling.
    pub word: String,
    /// Type variant documentation.
    pub description: String,
}
