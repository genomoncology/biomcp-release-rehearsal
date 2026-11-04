use crate::xml_element::Element;
use biomcp_fetch::{BadReply, Reply, xml::XmlReply};
use biomcp_types::{AssertionKind, VariantAssertion};
use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct Records {
    pub id: Option<String>,
    pub rows: Vec<VariantAssertion>,
    pub problems: Vec<biomcp_types::Problem>,
    pub refusals: crate::Refusal,
    pub accession: Option<String>,
    pub version: Option<String>,
}
impl Reply for Records {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
impl XmlReply for Records {
    fn decode_xml(xml: &str) -> Result<Self, BadReply> {
        let root = Element::root(xml, "ClinVarResult-Set")?;
        if root.child("ERROR")?.is_some() {
            return Err(BadReply);
        }
        let archives = root.children("VariationArchive")?;
        if archives.is_empty() {
            return Ok(Self {
                id: None,
                rows: Vec::new(),
                problems: Vec::new(),
                refusals: crate::Refusal::None,
                accession: None,
                version: None,
            });
        }
        if archives.len() != 1 {
            return Err(BadReply);
        }
        let archive = *archives.first().ok_or(BadReply)?;
        let id = archive.attr("VariationID")?;
        let accession = archive.attr("Accession")?;
        let version = archive.attr("Version")?;
        let classified = archive.child("ClassifiedRecord")?.ok_or(BadReply)?;
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::ClinVar,
            record: accession.as_ref().and_then(crate::mapped::record),
            list: Vec::new(),
        };
        let subject = problems.optional(
            "VariationID",
            Some(id.as_ref().ok_or(BadReply).and_then(|id| {
                format!("clinvar:{id}")
                    .parse::<biodata::VariantId>()
                    .map_err(|_| BadReply)
            })),
        );
        let subject = match subject {
            Some(id) => super::subject::map(archive, classified, id, &mut problems)?,
            None => None,
        };
        let Some(subject) = subject else {
            return Ok(Self {
                id,
                accession,
                version,
                rows: Vec::new(),
                problems: problems.list,
                refusals: crate::Refusal::Refused,
            });
        };
        let mut refusals = crate::Refusal::None;
        let rows = assertions(archive, classified, subject, &mut problems, &mut refusals)?;
        Ok(Self {
            id,
            rows,
            problems: problems.list,
            refusals,
            accession,
            version,
        })
    }
}

impl Records {
    pub(super) fn identity_problem(
        &self,
        id: &str,
        selected: Option<(&str, Option<&str>)>,
    ) -> Option<&'static str> {
        if self
            .problems
            .iter()
            .any(|problem| problem.field.as_ref() == "VariationID")
            || self.id.as_deref().is_some_and(|given| given != id)
        {
            return Some("VariationID");
        }
        if let Some((accession, version)) = selected.filter(|_| self.id.is_some()) {
            if self.accession.as_deref() != Some(accession) {
                return Some("Accession");
            }
            if version.is_some_and(|version| self.version.as_deref() != Some(version)) {
                return Some("Version");
            }
        }
        None
    }
}

fn assertions(
    archive: Element<'_>,
    classified: Element<'_>,
    subject: biodata::VariantRef,
    problems: &mut crate::mapped::Problems,
    refusals: &mut crate::Refusal,
) -> Result<Vec<VariantAssertion>, BadReply> {
    let mut rows = Vec::new();
    if let Some(class) = classified.child("Classifications")? {
        rows.extend(super::map::aggregate(
            archive,
            class,
            AssertionKind::Vcv,
            subject.clone(),
            Vec::new(),
            problems,
            refusals,
        )?);
    }
    if let Some(rcvs) = classified.child("RCVList")? {
        for rcv in rcvs.children("RCVAccession")? {
            if let Some(class) = rcv.child("RCVClassifications")? {
                problems.record = rcv.attr("Accession")?.and_then(crate::mapped::record);
                rows.extend(super::map::aggregate(
                    rcv,
                    class,
                    AssertionKind::Rcv,
                    subject.clone(),
                    super::detail::rcv_conditions(rcv, problems)?,
                    problems,
                    refusals,
                )?);
            }
        }
    }
    if let Some(scvs) = classified.child("ClinicalAssertionList")? {
        for scv in scvs.children("ClinicalAssertion")? {
            rows.extend(super::map::submission(
                scv,
                subject.clone(),
                problems,
                refusals,
            )?);
        }
    }
    Ok(rows)
}
