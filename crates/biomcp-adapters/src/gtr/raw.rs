use biodata::{GtrTestId, GtrTestType, GtrTestVersion};
use biomcp_fetch::{KeyKind, Keys, Row};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum Native {
    Version(Box<GtrTestVersion>),
    Relation(Relation),
    Refused(Refused),
}
/// Keep only actual native keys for a scoped identity refusal.
#[derive(Debug, Deserialize)]
pub(super) struct Refused {
    test_accession_ver: Option<crate::raw::Field<GtrTestId>>,
    #[serde(rename = "#accession_version")]
    accession: Option<crate::raw::Field<GtrTestId>>,
    object: Option<crate::raw::Field<Object>>,
    gene_symbol: Option<crate::raw::Field<Box<str>>>,
    #[serde(rename = "gene_or_SNOMED_CT_ID")]
    object_id: Option<crate::raw::Field<Box<str>>>,
    #[serde(rename = "GTR_identifier")]
    medgen: Option<crate::raw::Field<Box<str>>>,
    test_type: Option<crate::raw::Field<GtrTestType>>,
}
impl Refused {
    pub(super) fn matches(&self, q: &biomcp_types::DiagnosticSearch) -> bool {
        let object = self.object.as_ref().and_then(crate::raw::Field::as_ref);
        let symbol = self
            .gene_symbol
            .as_ref()
            .and_then(crate::raw::Field::as_ref);
        let gene_id = self.object_id.as_ref().and_then(crate::raw::Field::as_ref);
        let medgen = self.medgen.as_ref().and_then(crate::raw::Field::as_ref);
        q.test_type().is_none_or(|kind| {
            self.test_type.as_ref().and_then(crate::raw::Field::as_ref) == Some(&kind)
        }) && q.gene().is_none_or(|gene| {
            object == Some(&Object::Gene) && symbol.map(|s| &**s) == Some(gene.as_str())
        }) && q.gene_id().is_none_or(|gene| {
            object == Some(&Object::Gene) && gene_id.is_some_and(|id| **id == gene.to_string())
        }) && q.condition_id().is_none_or(|condition| {
            object == Some(&Object::Condition)
                && medgen.is_some_and(|id| format!("MedGen:{id}") == condition.as_str())
        })
    }
    pub(super) fn disclose(&self, cx: &mut crate::mapped::Problems) {
        let before = cx.list.len();
        let (field, id) = if self.test_accession_ver.is_some() {
            ("test_accession_ver", self.test_accession_ver.as_ref())
        } else {
            ("#accession_version", self.accession.as_ref())
        };
        let id = id.and_then(crate::raw::Field::as_ref);
        cx.record = id.and_then(crate::mapped::record);
        cx.required(field, id);
        if self.accession.is_some() {
            cx.required(
                "object",
                self.object.as_ref().and_then(crate::raw::Field::as_ref),
            );
        }
        if cx.list.len() == before {
            cx.required::<()>("record", None);
        }
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Relation {
    #[serde(rename = "#accession_version")]
    pub id: GtrTestId,
    pub test_type: Option<crate::raw::Field<GtrTestType>>,
    pub object: Object,
    #[serde(rename = "GTR_identifier")]
    pub medgen: Box<str>,
    #[serde(rename = "MIM_number")]
    pub mim: Box<str>,
    pub object_name: Box<str>,
    #[serde(rename = "gene_or_SNOMED_CT_ID")]
    pub object_id: Box<str>,
    pub gene_symbol: Box<str>,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Object {
    Gene,
    Condition,
}
impl Row for Native {
    fn check_part(&self, part: &str) -> Result<(), biomcp_fetch::BadReply> {
        if matches!(
            (self, part),
            (Self::Version(_), "versions") | (Self::Relation(_), "relations")
        ) || matches!((self, part),
            (Self::Refused(r), "versions") if r.test_accession_ver.is_some() && r.accession.is_none()
        ) || matches!((self, part),
            (Self::Refused(r), "relations") if r.accession.is_some() && r.test_accession_ver.is_none()
        ) {
            Ok(())
        } else {
            Err(biomcp_fetch::BadReply)
        }
    }
    fn check_bundle(&self, _rows: usize) -> Result<(), biomcp_fetch::BadReply> {
        // Optional native values are mapped with Problems on a scoped read.
        Ok(())
    }
    fn keys(&self) -> Keys {
        let mut exact = self
            .id()
            .map(|id| (KeyKind::Id, id.to_string().into()))
            .into_iter()
            .collect::<Vec<_>>();
        match self {
            Self::Version(_) => exact.push((KeyKind::Band, "versions".into())),
            Self::Refused(r) if r.test_accession_ver.is_some() => {
                exact.push((KeyKind::Band, "versions".into()));
            }
            _ => {}
        }
        let relation = match self {
            Self::Relation(r) => Some((
                Some(&r.object),
                Some(&r.gene_symbol),
                Some(&r.object_id),
                Some(&r.medgen),
                r.test_type.as_ref().and_then(crate::raw::Field::as_ref),
            )),
            Self::Refused(r) if r.accession.is_some() => Some((
                r.object.as_ref().and_then(crate::raw::Field::as_ref),
                r.gene_symbol.as_ref().and_then(crate::raw::Field::as_ref),
                r.object_id.as_ref().and_then(crate::raw::Field::as_ref),
                r.medgen.as_ref().and_then(crate::raw::Field::as_ref),
                r.test_type.as_ref().and_then(crate::raw::Field::as_ref),
            )),
            _ => None,
        };
        if let Some((object, symbol, object_id, medgen, kind)) = relation {
            exact.push((KeyKind::Band, "relations".into()));
            if let Some(kind) = kind {
                exact.push((KeyKind::Alias, kind.as_str().into()));
            }
            if object == Some(&Object::Gene) {
                if let Some(symbol) =
                    symbol.filter(|symbol| symbol.parse::<biodata::GeneSymbol>().is_ok())
                {
                    exact.push((KeyKind::Symbol, symbol.clone()));
                }
                if let Some(Ok(id)) = object_id.map(|id| id.parse::<biodata::NcbiGeneId>()) {
                    exact.push((KeyKind::Xref, format!("gene:{id}").into()));
                }
            } else if object == Some(&Object::Condition)
                && let Some(medgen) = medgen
                && format!("MedGen:{medgen}").parse::<biodata::Curie>().is_ok()
            {
                exact.push((KeyKind::Xref, format!("MedGen:{medgen}").into()));
            }
        }
        Keys {
            exact,
            words: Vec::new(),
        }
    }
}
impl Native {
    pub(super) fn id(&self) -> Option<&GtrTestId> {
        match self {
            Self::Version(v) => Some(&v.test_accession_ver),
            Self::Relation(v) => Some(&v.id),
            Self::Refused(v) => v
                .accession
                .as_ref()
                .or(v.test_accession_ver.as_ref())
                .and_then(crate::raw::Field::as_ref),
        }
    }
}
