//! HGNC identity resolution preserving source ambiguity and withdrawn identity.
mod resolve;
pub(crate) use resolve::resolve;

pub(crate) mod get;
mod next;
pub(crate) mod search;

mod sections;
