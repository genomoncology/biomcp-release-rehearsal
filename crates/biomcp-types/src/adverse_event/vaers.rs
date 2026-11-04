//! CDC WONDER retains native aggregate cells and independent table metadata.
use crate::ResponseBody;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

biodata::wire_enum! { described
    /// One independent CDC WONDER grouping.
    VaersSection {
        /// Native reported symptom rows.
        Symptoms => "symptoms",
        /// Reports classified by native seriousness.
        Seriousness => "seriousness",
        /// Native age groups.
        Age => "age",
    }
}
impl VaersSection {
    /// Native grouping code.
    pub fn code(self) -> &'static str {
        match self {
            Self::Symptoms => "D8.V13-level2",
            Self::Seriousness => "D8.V10",
            Self::Age => "D8.V1",
        }
    }
}
/// Published native influenza types, each selecting an independent table.
/// Labels preserve the publisher's spelling and qualifications.
pub const VAERS_INFLUENZA: &[(&str, &str)] = &[
    ("FLU(H1N1)", "INFLUENZA (H1N1) MONOVALENT (INJECTED)"),
    (
        "FLUN(H1N1)",
        "INFLUENZA (H1N1) MONOVALENT, (INTRANASAL SPRAY)",
    ),
    ("FLUX(SEASONAL)", "INFLUENZA VIRUS VACCINE, NO BRAND NAME"),
    (
        "FLU4(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, QUADRIVALENT (INJECTED)",
    ),
    (
        "FLUN4(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, QUADRIVALENT (INTRANASAL SPRAY)",
    ),
    (
        "FLUA4(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, QUADRIVALENT, ADJUVANT (INJECTED)",
    ),
    (
        "FLUC4(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, QUADRIVALENT, CELL-CULTURE-DERIVED (INJECTED)",
    ),
    (
        "FLUR4(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, QUADRIVALENT, RECOMBINANT (INJECTED)",
    ),
    (
        "FLU3(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, TRIVALENT (INJECTED)",
    ),
    (
        "FLUN3(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, TRIVALENT (INTRANASAL SPRAY)",
    ),
    (
        "FLUA3(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, TRIVALENT, ADJUVANT (INJECTED)",
    ),
    (
        "FLUC3(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, TRIVALENT, CELL-CULTURE-DERIVED (INJECTED)",
    ),
    (
        "FLUR3(SEASONAL)",
        "INFLUENZA VIRUS VACCINE, TRIVALENT, RECOMBINANT (INJECTED)",
    ),
    (
        "FLUX(H1N1)",
        "INFLUENZA(H1N1) MONOVALENT, UNKNOWN MANUFACTURER",
    ),
];
/// Resolve only exact supported vaccine codes or existing display names, ignoring case.
pub fn vaers_vaccine_code(text: &str) -> Option<&'static str> {
    if text.eq_ignore_ascii_case("MMR") {
        Some("MMR")
    } else if text.eq_ignore_ascii_case("COVID19") || text.eq_ignore_ascii_case("COVID-19 vaccine")
    {
        Some("COVID19")
    } else {
        VAERS_INFLUENZA
            .iter()
            .find_map(|(code, _)| text.eq_ignore_ascii_case(code).then_some(*code))
    }
}
/// Original native XML attributes, including label, value, displayed footer and suppression.
pub type VaersCell = BTreeMap<Box<str>, Box<str>>;
/// One whole native row, never reconstructed from another grouping.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VaersRow {
    /// Native label cell; no code is invented when absent.
    pub label: VaersCell,
    #[doc = " Native Events Reported count for this row. A footer counts occurrences, separate from \
        the message report total; suppressed and displayed-total text stay unchanged."]
    pub reports: VaersCell,
    /// Native Percent cell, retaining values above one hundred percent.
    pub percent: VaersCell,
}
/// Applied source text, retaining its original content and source role.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VaersNotice {
    /// Message, footnote or evaluated caveat.
    pub kind: Box<str>,
    /// Original source text, including embedded publisher markup.
    pub text: Box<str>,
}
/// Fixed native geographic selection, without a caller-selectable geography filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VaersScope {
    /// Native variable attributes, including source name and code.
    pub variable: VaersCell,
    /// Exact selected native value and its own catalog label.
    pub value: VaersCell,
}
/// Bounded page of one independent native vaccine summary table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VaersSummary {
    /// Native selected vaccine attributes.
    pub vaccine: VaersCell,
    /// Native fixed State / Territory selection.
    pub scope: VaersScope,
    /// Native grouping attributes.
    pub grouping: VaersCell,
    /// Native dataset attributes including processed-through vintage.
    pub dataset: VaersCell,
    /// Source measure definitions, including original names and codes.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub measures: Box<[VaersCell]>,
    /// Response-specific measure metadata without denominator inference.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub measure_metadata: Box<[VaersCell]>,
    /// Native report-total message, separate from the footer aggregation.
    pub report_total: Box<str>,
    /// Whole native footer cells, never a sum calculated by this program.
    pub footer: VaersRow,
    /// Native row order with local paging.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub rows: Box<[VaersRow]>,
    /// Applied native messages, evaluated caveats and footnotes.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub notices: Box<[VaersNotice]>,
    /// Reviewed interpretation boundary.
    pub interpretation: Box<str>,
}
impl ResponseBody for VaersSummary {
    const SCHEMA: &'static str = "biomcp.vaers-summary/5";
    type Query = super::AdverseEventSearch;
}
