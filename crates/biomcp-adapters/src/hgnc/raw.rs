use biodata::{Cytoband, EnsemblGeneId, GeneSymbol, HgncId, NcbiGeneId, UniProtAcc};

use biomcp_fetch::{KeyKind, Keys, Row};
use serde::Deserialize;

pub(super) fn cell(value: Option<&str>) -> &str {
    value.unwrap_or("")
}

#[derive(Debug, Deserialize)]
pub(super) struct Gene {
    pub hgnc_id: Option<String>,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub locus_group: Option<String>,
    pub locus_type: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
    pub alias_symbol: Option<String>,
    pub prev_symbol: Option<String>,
    pub entrez_id: Option<String>,
    pub ensembl_gene_id: Option<String>,
    pub uniprot_ids: Option<String>,
    pub omim_id: Option<String>,
    pub mane_select: Option<String>,
}
pub(super) fn parts(value: &str) -> impl Iterator<Item = &str> {
    value.split('|').filter(|v| !v.is_empty())
}
impl Row for Gene {
    fn keys(&self) -> Keys {
        let mut keys = identity(cell(self.hgnc_id.as_deref()), cell(self.symbol.as_deref()));
        for (kind, values) in [
            (KeyKind::PreviousSymbol, cell(self.prev_symbol.as_deref())),
            (KeyKind::Alias, cell(self.alias_symbol.as_deref())),
        ] {
            keys.exact.extend(parts(values).map(|v| (kind, v.into())));
            keys.words.extend(parts(values).map(Into::into));
        }
        if let Ok(id) = cell(self.entrez_id.as_deref()).parse::<NcbiGeneId>() {
            keys.exact
                .push((KeyKind::Xref, format!("entrez:{id}").into()));
        }
        if let Ok(id) = cell(self.ensembl_gene_id.as_deref()).parse::<EnsemblGeneId>() {
            keys.exact
                .push((KeyKind::Xref, format!("ensembl:{}", id.base()).into()));
        }
        for value in parts(cell(self.uniprot_ids.as_deref())) {
            if let Ok(id) = UniProtAcc::new(value) {
                keys.exact
                    .push((KeyKind::Xref, format!("uniprot:{id}").into()));
            }
        }
        if Cytoband::new(cell(self.location.as_deref())).is_ok() {
            keys.exact
                .push((KeyKind::Band, cell(self.location.as_deref()).into()));
        }
        keys.words.push(cell(self.name.as_deref()).into());
        keys
    }
}
fn identity(id: &str, symbol: &str) -> Keys {
    let mut exact = Vec::new();
    if let Ok(id) = id.parse::<HgncId>() {
        exact.push((KeyKind::Id, id.to_string().into()));
    }
    if GeneSymbol::new(symbol).is_ok() {
        exact.push((KeyKind::Symbol, symbol.into()));
    }
    Keys {
        exact,
        words: vec![symbol.into()],
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Withdrawn {
    #[serde(rename = "HGNC_ID")]
    pub hgnc_id: Option<String>,
    #[serde(rename = "STATUS")]
    pub status: Option<String>,
    #[serde(rename = "WITHDRAWN_SYMBOL")]
    pub symbol: Option<String>,
    #[serde(rename = "MERGED_INTO_REPORT(S) (i.e HGNC_ID|SYMBOL|STATUS)")]
    pub merged: Option<String>,
}
impl Row for Withdrawn {
    fn keys(&self) -> Keys {
        identity(cell(self.hgnc_id.as_deref()), cell(self.symbol.as_deref()))
    }
}
