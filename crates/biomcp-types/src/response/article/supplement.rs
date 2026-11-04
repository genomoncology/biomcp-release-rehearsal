use crate::{License, Pmcid, Section, Text};
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// Supplement format from an explicit JATS MIME declaration, with suffix fallback.
    FileKind, SourceUnknown {
        /// Excel workbook.
        Xlsx => "xlsx",
        /// Comma-separated text.
        Csv => "csv",
        /// Tab-separated text.
        Tsv => "tsv",
        /// Plain text lines.
        Txt => "txt",
        /// PDF download offered without parsing.
        Pdf => "pdf",
        /// Other source format.
        Other => "other",
    }
}
impl FileKind {
    /// Whether the local reader supports this format.
    pub fn readable(self) -> bool {
        matches!(self, Self::Xlsx | Self::Csv | Self::Tsv | Self::Txt)
    }
}
/// A JATS supplement confirmed by the same processing record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SupplementFile {
    /// Exact source filename.
    pub name: Text,
    /// Checked format.
    pub kind: FileKind,
    /// JATS label when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
    /// JATS caption when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<crate::Markdown>,
}
/// Available supplements with their owning article license.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Supplements {
    /// Article license from the record.
    pub license: License,
    /// Exact JATS/record intersection.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub files: Box<[SupplementFile]>,
}
/// One original sparse row with its numbered first source row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SheetRow {
    /// Original sheet name.
    pub sheet: Text,
    /// Original one-based row number.
    pub row: u32,
    /// First original row, including empty cells; never inferred column headings.
    pub first_row: FirstSheetRow,
    /// This row's cells, including empty cells before populated columns.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub cells: Box<[String]>,
    /// Sparse source cell fragment when the complete row exceeds the output bound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment: Option<CellFragment>,
}
/// The first original row without interpreting its cells as column headings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FirstSheetRow {
    /// Original one-based row number.
    pub row: u32,
    /// Original cells, empty when represented by cell fragments.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub cells: Box<[String]>,
}
/// A bounded portion of one original first-row or selected-row cell.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellFragment {
    /// Whether the cell belongs to the first source row.
    pub first_row: bool,
    /// Original one-based column.
    pub column: u32,
    /// Unicode character offset within this cell.
    pub offset: u32,
    /// Original cell text at this offset.
    pub text: String,
}
/// Source sheet with its maximum original row number.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SheetInfo {
    /// Sheet name.
    pub name: Text,
    /// Last original row number, preserving gaps.
    pub rows: u32,
}
/// Downloaded bounded file contents or an unsupported format's original URL.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SupplementContent {
    /// Explicit original-byte save receipt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved: Option<crate::ArticleSaved>,
    /// Article license.
    pub license: License,
    /// Hash of the original downloaded bytes; absent when no download occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<Text>,
    /// Unsupported file URL, returned without downloading it.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Option<Text>,
    /// Typed unsupported reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<crate::Reason>,
    /// Original sheet dimensions.
    #[serde(skip_serializing_if = "biodata::absent_or_empty")]
    pub sheets: Option<Box<[SheetInfo]>>,
    /// Selected original rows.
    #[serde(skip_serializing_if = "biodata::absent_or_empty")]
    pub rows: Option<Box<[SheetRow]>>,
    /// Explicit absence of matching text from the returned search page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_message: Option<String>,
    /// Explicit variant spellings searched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forms: Option<[String; 4]>,
}
/// One supplement reader answer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleSupplementResult {
    /// Resolved PMC identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<Pmcid>,
    /// Independently licensed supplement content.
    pub content: Section<SupplementContent>,
}
impl super::super::ResponseBody for ArticleSupplementResult {
    const SCHEMA: &'static str = "biomcp.article-supplement/7";
    type Query = crate::ArticleSupplement;
}
