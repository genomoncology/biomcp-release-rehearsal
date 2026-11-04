//! Read command identities from the shared declaration.
pub(crate) use biomcp_types::command_paths;
pub(crate) fn query_schema(request: &biomcp_types::Request) -> &'static str {
    request.schema()
}
