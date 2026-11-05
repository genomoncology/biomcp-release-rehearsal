//! Publisher display names come from the owning adapter.
/// Source label used beside an untitled native record.
pub fn display_name(source: biomcp_types::Source) -> &'static str {
    crate::decl::decl(source).display
}

/// Source-owned short publisher spellings used in discovery prose.
pub fn discovery_aliases(source: biomcp_types::Source) -> &'static [&'static str] {
    crate::decl::decl(source).discovery_aliases
}
