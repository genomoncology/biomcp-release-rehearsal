//! Typed catalog ceilings are configurable without changing clinical membership.
use super::Settings;

pub(crate) const DEFAULT_TYPED_TOOL_LIMIT: usize = 20;
pub(crate) const DEFAULT_TYPED_DEFINITION_BYTES: usize = 128 * 1024;
pub(crate) const DEFAULT_TYPED_OUTPUT_DEFINITION_BYTES: usize = 192 * 1024;

pub(crate) fn mcp_typed_budget(settings: &Settings) -> (usize, usize) {
    let file = settings.as_ref().ok().and_then(Option::as_ref);
    let byte_default = if super::mcp_typed_schema_options(settings).0 {
        DEFAULT_TYPED_OUTPUT_DEFINITION_BYTES
    } else {
        DEFAULT_TYPED_DEFINITION_BYTES
    };
    (
        file.and_then(|file| file.mcp_typed_tool_limit)
            .map_or(DEFAULT_TYPED_TOOL_LIMIT, std::num::NonZeroUsize::get),
        file.and_then(|file| file.mcp_typed_definition_bytes)
            .map_or(byte_default, std::num::NonZeroUsize::get),
    )
}
