use crate::mapped::Problems;
use crate::xml_element::Element;

use biomcp_fetch::BadReply;
use biomcp_types::Capped;
use biomcp_types::{Indexing, MeshHeading, MeshQualifier};
pub(super) fn parse(
    citation: Element<'_>,
    row: Element<'_>,
    problems: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Result<Indexing, BadReply> {
    let mut mesh = Vec::new();
    if let Some(list) = citation.child("MeshHeadingList")? {
        for heading in list.children("MeshHeading")? {
            let Some(descriptor) = heading.child("DescriptorName")? else {
                *refusals = crate::Refusal::Refused;
                problems.required::<()>("mesh.descriptor", None);
                continue;
            };
            let Some(name) = problems.text("mesh.descriptor", Some(&descriptor.text()?)) else {
                *refusals = crate::Refusal::Refused;
                continue;
            };
            let mut qualifiers = Vec::new();
            for qualifier in heading.children("QualifierName")? {
                if let Some(name) = problems.text("mesh.qualifiers.name", Some(&qualifier.text()?))
                {
                    qualifiers.push(MeshQualifier {
                        name,
                        major: major(qualifier)?,
                    });
                } else {
                    *refusals = crate::Refusal::Refused;
                }
            }
            mesh.push(MeshHeading {
                descriptor: name,
                major: major(descriptor)?,
                qualifiers: qualifiers.into(),
            });
        }
    }
    let mut keywords = Vec::new();
    for list in citation.children("KeywordList")? {
        for keyword in list.children("Keyword")? {
            if let Some(value) = problems.text("keywords", Some(&keyword.text()?)) {
                keywords.push(value);
            }
        }
    }
    let mut affiliations = Vec::new();
    let mut orcids = Vec::new();
    if let Some(list) = row.child("AuthorList")? {
        for author in list.children("Author")? {
            for info in author.children("AffiliationInfo")? {
                for affiliation in info.children("Affiliation")? {
                    if let Some(value) = problems.text("affiliations", Some(&affiliation.text()?)) {
                        affiliations.push(value);
                    }
                }
            }
            for identifier in author.children("Identifier")? {
                if identifier.attr("Source")?.as_deref() == Some("ORCID") {
                    let Some(id) = problems.checked_text("orcids", Some(&identifier.text()?))
                    else {
                        continue;
                    };
                    if !orcids.contains(&id) {
                        orcids.push(id);
                    }
                }
            }
        }
    }
    Ok(Indexing {
        mesh: mesh.into(),
        keywords: keywords.into(),
        affiliations: Capped {
            total: affiliations.len() as u64,
            items: affiliations.into_iter().take(10).collect(),
        },
        orcids: orcids.into(),
    })
}

fn major(node: Element<'_>) -> Result<bool, BadReply> {
    match node.attr("MajorTopicYN")?.as_deref() {
        Some("Y") => Ok(true),
        Some("N") => Ok(false),
        _ => Err(BadReply),
    }
}
