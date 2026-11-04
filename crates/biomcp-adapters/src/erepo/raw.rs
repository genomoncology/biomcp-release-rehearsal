//! Native summary documents preserve independently valid source fields.
use crate::native_text::NativeText;
use crate::{mapped::Problems, raw::Field};
use biodata::{Attribution, Text, VariantRef};
use biomcp_fetch::{BadReply, Reply};
use biomcp_types::{
    AssertionCondition, AssertionDetail, AssertionDomain, AssertionKind, Source, VariantAssertion,
};
use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct ReplyBody {
    pub data: Option<Vec<Field<Record>>>,
    status: Status,
}
#[derive(Deserialize)]
struct Status {
    code: u16,
    name: String,
    msg: Option<String>,
}
impl Reply for ReplyBody {
    fn check(&self) -> Result<(), BadReply> {
        match self.status.code {
            200 if self.data.is_some() => Ok(()),
            404 if self.status.name == "Not Found"
                && self.status.msg.as_deref() == Some("No records were found for given query")
                && self.data.as_ref().is_none_or(Vec::is_empty) =>
            {
                Ok(())
            }
            _ => Err(BadReply),
        }
    }
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        if status == self.status.code {
            self.check()
        } else {
            Err(BadReply)
        }
    }
}
#[derive(Deserialize)]
pub(super) struct Record {
    #[serde(rename = "caId")]
    pub ca_id: String,
    #[serde(rename = "PCERDocID")]
    pub id: String,
    #[serde(rename = "docVersion")]
    pub version: String,
    classification: Option<Field<NativeText>>,
    pub uuid: Option<Field<String>>,
    condition: Option<Field<NativeText>>,
    #[serde(rename = "mondoId")]
    mondo: Option<Field<biodata::Curie>>,
    ep: Option<Field<NativeText>>,
    gene: Option<Field<Text>>,
    #[serde(rename = "metCodes")]
    criteria: Option<Field<Vec<Field<Text>>>>,
    #[serde(rename = "unMetCodes")]
    unmet: Option<Field<Vec<Field<NativeText>>>>,
    #[serde(rename = "summaryDesc")]
    summary: Option<Field<NativeText>>,
    #[serde(rename = "approvedDate")]
    approved: Option<Field<biodata::PartialDate>>,
    #[serde(rename = "publishedDate")]
    published: Option<Field<biodata::PartialDate>>,
    #[serde(rename = "versionsList")]
    pub versions: Option<Field<Vec<Field<Text>>>>,
}
impl Record {
    pub(super) fn matches(&self, caid: &str) -> bool {
        self.ca_id == caid
            && self.ca_id.parse::<biodata::VariantId>().is_ok()
            && !self.id.trim().is_empty()
            && !self.version.trim().is_empty()
    }
    pub(super) fn map(mut self, caid: &str, cx: &mut Problems) -> Option<VariantAssertion> {
        cx.record = cx.text("PCERDocID", Some(&self.id));
        let id = cx.required("PCERDocID", cx.record.clone())?;
        let version = cx.text("docVersion", Some(&self.version))?;
        let allele = cx.optional("caId", Some(self.ca_id.parse::<biodata::VariantId>()))?;
        if self.ca_id != caid {
            cx.required::<()>("caId", None);
            return None;
        }
        let uuid = cx.field("uuid", self.uuid.take());
        let url = uuid.as_deref().and_then(|uuid| {
            cx.text(
                "uuid",
                Some(&format!(
                    "https://erepo.clinicalgenome.org/evrepo/ui/classification/{uuid}?version={}",
                    self.version
                )),
            )
        });
        let attribution = Attribution {
            approved: None,
            published: None,
            source: Source::Erepo.text(),
            record: Some(id),
            version: Some(version),
            release: None,
            submitter: cx.source_field("ep", self.ep.take()),
            evaluated: None,
            url,
        };
        let conditions = self.conditions(cx);
        let detail = AssertionDetail {
            conditions,
            summary: cx.source_field("summaryDesc", self.summary),
            unmet_criteria: cx
                .fields("unMetCodes", self.unmet)
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect(),
            published: cx.field("publishedDate", self.published),
            approved: cx.field("approvedDate", self.approved),
            versions: cx.fields("versionsList", self.versions),
            ..Default::default()
        };
        let classification = cx.source_field("classification", self.classification);
        let mut row = match classification {
            Some(label) => crate::assertion::row(
                AssertionKind::Erepo,
                AssertionDomain::Germline,
                label,
                attribution,
                VariantRef::allele(allele),
                detail,
                None,
            ),
            None => crate::assertion::missing(
                AssertionKind::Erepo,
                AssertionDomain::Germline,
                attribution,
                detail,
            ),
        };
        let criteria = cx.fields("metCodes", self.criteria);
        let gene = cx.field("gene", self.gene).and_then(|label| {
            cx.optional(
                "gene",
                Some(biodata::SourceGeneRef::new(label, None, None, None)),
            )
        });
        if let Some(statement) = &mut row.statement {
            statement.criteria = criteria;
            statement.gene_context = gene;
        } else if let Some(detail) = &mut row.detail {
            detail.met_criteria = criteria;
            detail.gene_context = gene;
        }
        Some(row)
    }
    fn conditions(&mut self, cx: &mut Problems) -> Box<[AssertionCondition]> {
        let condition = AssertionCondition {
            database: None,
            accession: None,
            id: cx.field("mondoId", self.mondo.take()),
            label: cx.source_field("condition", self.condition.take()),
        };
        if condition.id.is_some() || condition.label.is_some() {
            vec![condition].into()
        } else {
            Box::default()
        }
    }
}
