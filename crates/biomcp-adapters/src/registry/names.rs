//! Publisher display names come from the owning adapter.
/// Source label used beside an untitled native record.
pub fn display_name(source: biomcp_types::Source) -> &'static str {
    crate::decl::decl(source).display
}
