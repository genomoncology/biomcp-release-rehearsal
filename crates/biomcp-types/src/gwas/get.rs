use crate::QueryInvalid;
use biodata::{GwasAssociationId, GwasStudyId};
use serde::{Deserialize, Serialize};
biodata::wire_enum! {
    described
    /// Association-only section.
    GwasAssociationSection {
        #[doc = "Native locus and reported gene context."] Loci => "loci"
    }
}
biodata::wire_enum! {
    described
    /// Study-only sections.
    GwasStudySection {
        #[doc = "Shared association rows."] Associations => "associations",
        #[doc = "Independent native ancestry rows."] Ancestries => "ancestries"
    }
}
/// Explicit resource sum; impossible section and identity pairings fail during decoding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum GwasGet {
    /// Read an association.
    Association {
        /// Qualified positive association identity.
        id: GwasAssociationId,
        /// Optional native locus section.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        section: Option<GwasAssociationSection>,
        /// Checked independent section page.
        #[serde(flatten)]
        page: GwasSectionPage,
    },
    /// Read a study.
    Study {
        /// Exact Catalog accession.
        id: GwasStudyId,
        /// Optional study relation section.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        section: Option<GwasStudySection>,
        /// Checked independent section page.
        #[serde(flatten)]
        page: GwasSectionPage,
    },
}
/// Bounded relation window, independent from identity paging.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PageFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "PageFields"))]
pub struct GwasSectionPage(PageFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct PageFields {
    /// Maximum native rows.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    section_limit: u32,
    /// Rows to skip in this relation.
    #[serde(default)]
    section_offset: u32,
    /// Retained ancestry reply for local pages without provider access.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "biodata::Text"))]
    snapshot: Option<biodata::Text>,
}
fn limit() -> u32 {
    3
}
crate::query::validation::checked_query!(GwasSectionPage, PageFields);
impl TryFrom<PageFields> for GwasSectionPage {
    type Error = QueryInvalid;
    fn try_from(f: PageFields) -> Result<Self, Self::Error> {
        if !(1..=50).contains(&f.section_limit) {
            return Err(QueryInvalid::new(&["section_limit"]));
        }
        Ok(Self(f))
    }
}
impl GwasSectionPage {
    /// Selected relation row bound.
    pub fn limit(&self) -> u32 {
        self.0.section_limit
    }
    /// Selected relation offset.
    pub fn offset(&self) -> u32 {
        self.0.section_offset
    }
}
impl crate::Paged for GwasSectionPage {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.section_offset = offset;
    }
    fn clear_page_context(&mut self) {
        self.0.snapshot = None;
    }
}

impl Default for GwasSectionPage {
    fn default() -> Self {
        Self(PageFields {
            section_limit: 3,
            section_offset: 0,
            snapshot: None,
        })
    }
}
impl GwasSectionPage {
    /// Retained ancestry source reply.
    pub fn snapshot(&self) -> Option<&biodata::Text> {
        self.0.snapshot.as_ref()
    }
    /// Bind every local page to the same reply.
    pub fn with_snapshot(&self, token: biodata::Text) -> Self {
        let mut page = self.clone();
        page.0.snapshot = Some(token);
        page
    }
}
impl GwasGet {
    /// Exact study card with default relation paging.
    pub fn study(id: GwasStudyId) -> Self {
        Self::Study {
            id,
            section: None,
            page: GwasSectionPage::default(),
        }
    }
    /// Exact association card with default relation paging.
    pub fn association(id: GwasAssociationId) -> Self {
        Self::Association {
            id,
            section: None,
            page: GwasSectionPage::default(),
        }
    }
    /// Relation window for either source identity.
    pub fn page(&self) -> &GwasSectionPage {
        match self {
            Self::Association { page, .. } | Self::Study { page, .. } => page,
        }
    }
    /// Continue the checked identity and section.
    pub fn with_page(&self, page: GwasSectionPage) -> Self {
        let mut next = self.clone();
        match &mut next {
            Self::Association { page: p, .. } | Self::Study { page: p, .. } => *p = page,
        }
        next
    }
}
impl crate::Paged for GwasGet {
    type Offset = ();
    fn set_page_offset(&mut self, _offset: Self::Offset) {
        match self {
            Self::Association { page, .. } | Self::Study { page, .. } => {
                crate::Paged::set_page_offset(page, 0)
            }
        }
    }
    fn clear_page_context(&mut self) {
        match self {
            Self::Association { page, .. } | Self::Study { page, .. } => page.0.snapshot = None,
        }
    }
}

/// Exact association request fields for the shared tagged request union.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasAssociationGet {
    /// Qualified source association identity.
    pub id: GwasAssociationId,
    /// Association-only relation selector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<GwasAssociationSection>,
    /// Independent checked relation window.
    #[serde(flatten)]
    pub page: GwasSectionPage,
}
/// Exact study request fields for the shared tagged request union.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasStudyGet {
    /// Exact Catalog accession.
    pub id: GwasStudyId,
    /// Study-only relation selector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<GwasStudySection>,
    /// Independent checked relation window.
    #[serde(flatten)]
    pub page: GwasSectionPage,
}
impl From<GwasAssociationGet> for GwasGet {
    fn from(q: GwasAssociationGet) -> Self {
        Self::Association {
            id: q.id,
            section: q.section,
            page: q.page,
        }
    }
}
impl From<GwasStudyGet> for GwasGet {
    fn from(q: GwasStudyGet) -> Self {
        Self::Study {
            id: q.id,
            section: q.section,
            page: q.page,
        }
    }
}
impl From<GwasGet> for crate::Request {
    fn from(q: GwasGet) -> Self {
        match q {
            GwasGet::Association { id, section, page } => {
                Self::GetGwasAssociation(GwasAssociationGet { id, section, page })
            }
            GwasGet::Study { id, section, page } => {
                Self::GetGwasStudy(GwasStudyGet { id, section, page })
            }
        }
    }
}

mod checked;
