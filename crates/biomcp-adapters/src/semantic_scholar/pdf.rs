//! The paper record supplies a link, never credentials for the publisher.

use crate::{mapped::Problems, raw::Field};
use biodata::License;
use biomcp_fetch::Input;
use biomcp_fetch::{Arg, BadReply, Leg, Reply, RequestError};
use biomcp_types::ArticleInput;
use biomcp_types::OpenAccessStatus;
use serde::Deserialize;
/// Source PDF-link fields with absence preserved.
#[derive(Debug, Deserialize)]
#[serde(from = "RawRecord")]
pub struct PdfRecord {
    /// The source explicitly reports a missing link with null.
    pub open_access_pdf: Option<PdfLink>,
    /// Rejected source access metadata.
    pub problems: Vec<biomcp_types::Problem>,
}
/// Publisher URL and source-provided access metadata.
#[derive(Debug)]
pub struct PdfLink {
    /// Publisher URL; the transport validates it before use.
    pub url: String,
    /// Open access status, normalized from the provider's uppercase spelling.
    pub status: OpenAccessStatus,
    /// Provider license label, which may be absent.
    pub license: Option<String>,
}
impl Reply for PdfRecord {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRecord {
    open_access_pdf: Option<Field<RawLink>>,
}
#[derive(Deserialize)]
struct RawLink {
    url: String,
    status: Option<Field<Access>>,
    license: Option<Field<String>>,
}
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Access(OpenAccessStatus);
impl TryFrom<String> for Access {
    type Error = biodata::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.to_lowercase().parse().map(Self)
    }
}
impl From<RawRecord> for PdfRecord {
    fn from(raw: RawRecord) -> Self {
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: None,
            list: Vec::new(),
        };
        let open_access_pdf = p
            .field("open_access_pdf", raw.open_access_pdf)
            .and_then(|link| {
                let status = p
                    .required_field("open_access_pdf.status", link.status)
                    .map(|v| v.0);
                let malformed_license = matches!(link.license, Some(Field::Invalid(_)));
                let license = p.field("open_access_pdf.license", link.license);
                if malformed_license {
                    return None;
                }
                Some(PdfLink {
                    url: link.url,
                    status: status?,
                    license,
                })
            });
        Self {
            open_access_pdf,
            problems: p.list,
        }
    }
}
impl PdfLink {
    /// Preserve absent and unknown license labels without asserting reuse rights.
    pub fn license(&self) -> License {
        match self
            .license
            .as_deref()
            .unwrap_or("")
            .to_ascii_uppercase()
            .as_str()
        {
            "CCBY" | "CC-BY" => License::CcBy,
            "CCBYNC" | "CC-BY-NC" => License::CcByNc,
            "CCBYSA" | "CC-BY-SA" => License::CcBySa,
            "CCBYNCSA" | "CC-BY-NC-SA" => License::CcByNcSa,
            "CCBYND" | "CC-BY-ND" => License::CcByNd,
            "CCBYNCND" | "CC-BY-NC-ND" => License::CcByNcNd,
            "CC0" => License::Cc0,
            _ => License::Other,
        }
    }
}
/// Read only the open-access PDF fields using the shared paper path and API quota.
pub fn pdf(leg: &Leg<'_>, input: &ArticleInput) -> Result<PdfRecord, RequestError> {
    leg.call(
        &super::paper_endpoint(input)?,
        Input::Params(&[("fields", Arg::Text("openAccessPdf"))]),
        biomcp_fetch::Read::json(),
    )
    .map(|received| received.value)
}

/// Credential-free publisher declaration, distinct from the optional-key API.
pub static PUBLISHER: biomcp_fetch::SourceSpec = biomcp_fetch::SourceSpec {
    destinations: biomcp_fetch::Destinations::PublisherPdf,
    user_agent: None,
    source: biomcp_types::Source::SemanticScholar,
    terms: biomcp_fetch::Terms {
        name: "Publisher PDF; article-specific source license",
        url: "https://api.semanticscholar.org/api-docs/graph#tag/Paper-Data",
        usage: biomcp_types::Use::PerRecord,
    },
    base: "",
    key: biomcp_fetch::KeyUse::None,
    quota: &crate::quotas::PUBLISHER_PDF,
    deadline: std::time::Duration::from_secs(15),
};
