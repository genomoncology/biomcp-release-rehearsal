//! Data-only presentation hints; paths refer exclusively to response JSON.
use biomcp_types::ResponseBody;
pub(super) struct Hint {
    pub(super) schema: &'static str,
    pub(super) title: &'static str,
    pub(super) columns: &'static [&'static str],
}
const HINTS: &[Hint] = &[
    Hint {
        schema: biomcp_types::DiseaseSites::SCHEMA,
        title: "",
        columns: &["code", "label", "active"],
    },
    Hint {
        schema: biomcp_types::ArticleSupplementResult::SCHEMA,
        title: "",
        columns: &["sheet", "row", "header", "cells", "fragment"],
    },
    Hint {
        schema: biomcp_types::GeneCards::SCHEMA,
        title: "/answer/gene/symbol",
        columns: &[],
    },
    Hint {
        schema: biomcp_types::ArticleCards::SCHEMA,
        title: "",
        columns: &[],
    },
    Hint {
        schema: biomcp_types::GenePage::SCHEMA,
        title: "",
        columns: &["symbol", "name", "hgnc_id", "location", "status"],
    },
    Hint {
        schema: biomcp_types::ArticlePage::SCHEMA,
        title: "",
        columns: &[
            "pmid",
            "header/title",
            "header/journal",
            "header/date",
            "found_by",
            "cited_by",
        ],
    },
    Hint {
        schema: biomcp_types::ArticleEdgePage::SCHEMA,
        title: "",
        columns: &["pmid", "header/title", "header/journal", "cited_by"],
    },
];
pub(super) fn find(schema: &str) -> Option<&'static Hint> {
    HINTS.iter().find(|hint| hint.schema == schema)
}
