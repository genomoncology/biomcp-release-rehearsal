use biomcp_fetch::{BadReply, Reply, xml::XmlReply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(rename = "ListBucketResult")]
pub(super) struct Listing {
    #[serde(rename = "Contents")]
    pub contents: Option<Vec<Entry>>,
    #[serde(rename = "IsTruncated")]
    truncated: bool,
}
#[derive(Debug, Deserialize)]
pub(super) struct Entry {
    #[serde(rename = "Key")]
    pub key: String,
}
impl Reply for Listing {
    fn check(&self) -> Result<(), BadReply> {
        if self.truncated {
            Err(BadReply)
        } else {
            Ok(())
        }
    }
}
impl XmlReply for Listing {}
#[derive(Debug, Deserialize)]
pub(super) struct Record {
    pub pmcid: biodata::Pmcid,
    pub pmid: Option<u32>,
    pub license_code: Option<crate::raw::Field<String>>,
    pub is_manuscript: Option<crate::raw::Field<bool>>,
    pub xml_url: String,
    pub media_urls: Option<crate::raw::Field<Vec<crate::raw::Field<String>>>>,
}
impl Reply for Record {
    fn check(&self) -> Result<(), BadReply> {
        if self.pmid == Some(0) {
            Err(BadReply)
        } else {
            Ok(())
        }
    }
}
