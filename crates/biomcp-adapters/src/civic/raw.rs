use crate::raw::Field;

use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Genes<const N: usize> {
    pub data: Aliases,
    #[serde(flatten)]
    extra: std::collections::BTreeMap<String, serde::de::IgnoredAny>,
}
pub(super) type Aliases = crate::graphql::Aliases<Gene>;
impl<const N: usize> Reply for Genes<N> {
    fn check(&self) -> Result<(), BadReply> {
        if self.extra.contains_key("errors") {
            return Err(BadReply);
        }
        self.data.check::<N>()?;
        Ok(())
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Gene {
    pub id: Option<Field<u64>>,
    #[serde(rename = "entrezId")]
    pub entrez_id: Option<Field<u32>>,
    pub description: Option<Field<String>>,
    pub link: Option<Field<String>>,
    pub variants: Option<Field<Variants>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Variants {
    #[serde(rename = "totalCount")]
    pub total_count: Option<Field<u64>>,
}
