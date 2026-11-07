//! Record-owned permissions shared by article adapters and retained replies.
mod abstracts;
mod lookup;
mod policy;
pub(crate) mod pubmed;
pub(crate) mod pubtator;
pub(crate) mod semantic;
pub use lookup::{Licenses, licenses};
pub(crate) use policy::{europepmc, litsense, permitted, unchecked, withheld, withheld_item};
use policy::{is_abstract, pmid};
pub use policy::{
    item_unchecked, items, return_unchecked, snapshot, take_unchecked, text_unchecked,
    text_withheld, unchecked_notice,
};
