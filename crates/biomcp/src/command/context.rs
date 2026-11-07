//! One MCP call's process settings, shared transport, credentials and lifetime.
use crate::{config::StartupError, grammar::Globals};
use biomcp_fetch::{Cancellation, Fetch, Secrets};
use biomcp_types::Source;
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Copy)]
pub(crate) struct Context<'a> {
    pub(crate) globals: &'a Globals,
    pub(crate) root: &'a crate::grammar::Root,
    pub(crate) settings: &'a crate::config::Settings,
    pub(crate) fetch: &'a Result<Arc<Fetch>, StartupError>,
    pub(crate) recovery: Option<&'a crate::mcp::Recovery>,
    pub(crate) secrets: Option<&'a Secrets>,
    pub(crate) excluded: Option<&'a BTreeSet<Source>>,
    pub(crate) declared_use: Option<super::DeclaredUse>,
    pub(crate) scope: Option<&'a Cancellation>,
}
impl<'a> Context<'a> {
    pub(crate) fn new(
        globals: &'a Globals,
        fetch: &'a Result<Arc<Fetch>, StartupError>,
        root: &'a crate::grammar::Root,
        settings: &'a crate::config::Settings,
    ) -> Self {
        Self {
            globals,
            root,
            settings,
            fetch,
            recovery: None,
            secrets: None,
            excluded: None,
            declared_use: None,
            scope: None,
        }
    }
}
