//! Canonical record pages belong to the source registry, never the projector.
use biomcp_types::Source;

/// A publisher's native identity namespace within one source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordKind {
    /// Exact provider author identity.
    Author,
    /// Exact Cellosaurus primary accession.
    CellLine,
    /// Native pathway stable ID.
    Pathway,
    /// Native study accession.
    Study,
    /// Native association ID.
    Association,
    /// Native adverse-event report ID.
    AdverseEvent,
    /// Native trial ID.
    Trial,
    /// Native diagnostic product or submission ID.
    Diagnostic,
    /// Native drug product reference.
    Drug,
    /// Source gene identity.
    Gene,
}

/// One source-owned link policy for one native identity kind.
#[derive(Debug)]
pub struct RecordDeclaration {
    /// Registered source owning the identifier.
    pub source: Source,
    /// Native ID namespace, independent of the command's entity name.
    pub kind: RecordKind,
    /// Publisher display name used when a native title is absent.
    pub display_name: &'static str,
    /// Verified publisher record URL; exactly one `{id}` substitution.
    /// Absence keeps the row uncitable unless it carries its own native URL.
    pub template: Option<&'static str>,
}

/// Exhaustive declared policies for the otherwise URL-less research shapes.
/// Publisher witnesses are retained by the bounded native record URL proof.
pub const DECLARATIONS: &[RecordDeclaration] = &[
    RecordDeclaration {
        source: Source::Orcid,
        kind: RecordKind::Author,
        display_name: "ORCID record",
        template: Some("https://orcid.org/{id}"),
    },
    RecordDeclaration {
        source: Source::SemanticScholar,
        kind: RecordKind::Author,
        display_name: "Semantic Scholar author",
        template: Some("https://www.semanticscholar.org/author/{id}"),
    },
    RecordDeclaration {
        source: Source::Cellosaurus,
        kind: RecordKind::CellLine,
        display_name: "Cellosaurus",
        template: Some("https://www.cellosaurus.org/{id}"),
    },
    RecordDeclaration {
        source: Source::Reactome,
        kind: RecordKind::Pathway,
        display_name: "Reactome pathway",
        template: Some("https://reactome.org/content/detail/{id}"),
    },
    RecordDeclaration {
        source: Source::WikiPathways,
        kind: RecordKind::Pathway,
        display_name: "WikiPathways pathway",
        template: Some("https://www.wikipathways.org/pathways/{id}.html"),
    },
    RecordDeclaration {
        source: Source::Cbioportal,
        kind: RecordKind::Study,
        display_name: "cBioPortal study",
        template: Some("https://www.cbioportal.org/study/summary?id={id}"),
    },
    RecordDeclaration {
        source: Source::Gwas,
        kind: RecordKind::Association,
        display_name: "GWAS Catalog association",
        template: Some("https://www.ebi.ac.uk/gwas/rest/api/v2/associations/{id}"),
    },
    RecordDeclaration {
        source: Source::Gwas,
        kind: RecordKind::Study,
        display_name: "GWAS Catalog study",
        template: Some("https://www.ebi.ac.uk/gwas/rest/api/v2/studies/{id}"),
    },
    RecordDeclaration {
        source: Source::Faers,
        kind: RecordKind::AdverseEvent,
        display_name: "FDA FAERS report",
        template: Some(
            "https://api.fda.gov/drug/event.json?search=safetyreportid%3A%22{id}%22&limit=5&skip=0&sort=receivedate%3Adesc",
        ),
    },
    RecordDeclaration {
        source: Source::Maude,
        kind: RecordKind::AdverseEvent,
        display_name: "FDA MAUDE report",
        template: Some(
            "https://api.fda.gov/device/event.json?search=mdr_report_key%3A%22{id}%22&limit=1",
        ),
    },
    RecordDeclaration {
        source: Source::Nci,
        kind: RecordKind::Trial,
        display_name: "NCI trial",
        template: Some(
            "https://www.cancer.gov/research/participate/clinical-trials-search/v?id={id}",
        ),
    },
    RecordDeclaration {
        source: Source::FdaDrugRecall,
        kind: RecordKind::Drug,
        display_name: "FDA drug recall",
        template: Some(
            "https://api.fda.gov/drug/enforcement.json?search=recall_number.exact%3A%22{id}%22&limit=1&skip=0",
        ),
    },
    RecordDeclaration {
        source: Source::FdaDeviceRecall,
        kind: RecordKind::Diagnostic,
        display_name: "FDA medical device recall",
        template: Some(
            "https://api.fda.gov/device/recall.json?search=product_res_number.exact%3A%22{id}%22&limit=1&skip=0",
        ),
    },
    RecordDeclaration {
        source: Source::Fda510k,
        kind: RecordKind::Diagnostic,
        display_name: "FDA 510(k) submission",
        template: None,
    },
    RecordDeclaration {
        source: Source::WhoPq,
        kind: RecordKind::Drug,
        display_name: "WHO prequalified medicine",
        template: None,
    },
    RecordDeclaration {
        source: Source::WhoIvd,
        kind: RecordKind::Diagnostic,
        display_name: "WHO prequalified diagnostic",
        template: None,
    },
    RecordDeclaration {
        source: Source::Cpic,
        kind: RecordKind::Gene,
        display_name: "CPIC gene",
        template: None,
    },
];

/// Look up the sole policy for one registered source and native ID kind.
pub fn declaration(source: Source, kind: RecordKind) -> Option<&'static RecordDeclaration> {
    DECLARATIONS
        .iter()
        .find(|row| row.source == source && row.kind == kind)
}

/// Substitute an opaque identifier without letting it change URL structure.
pub fn record_url(source: Source, kind: RecordKind, id: &str) -> Option<String> {
    if id.is_empty() || id == "." || id == ".." || id.chars().any(char::is_control) {
        return None;
    }
    let template = declaration(source, kind)?.template?;
    let mut encoded = String::new();
    for byte in id.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(&mut encoded, "%{byte:02X}").ok()?;
        }
    }
    Some(template.replace("{id}", &encoded))
}
