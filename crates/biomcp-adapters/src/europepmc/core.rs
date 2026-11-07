//! Checked Europe PMC core records and their JATS document boundary.
use super::SPEC;
use crate::native_text::NativeText;
use crate::{
    Mapped,
    mapped::{Problems, record as provenance},
    raw::Field,
};
use crate::{jats, pmc_oa};
use biodata::{Doi, EuropePmcId, PartialDate, Pmid, Text};
use biomcp_fetch::Input;

use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, Leg, Reply, ReplyMetadata, RequestError};
use biomcp_types::{AbstractPart, Article};
use serde::Deserialize;
const CORE: Endpoint = SPEC.get("/").fresh(Freshness::Days(1));
const FULLTEXT: Endpoint = SPEC.get("/").fresh(Freshness::Days(7));
#[derive(Deserialize)]
struct Raw {
    result: Record,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    id: EuropePmcId,
    source: String,
    pmid: Option<Field<Pmid>>,
    doi: Option<Field<Doi>>,
    title: Option<Field<NativeText>>,
    first_publication_date: Option<Field<PartialDate>>,
    author_list: Option<Field<Authors>>,
    abstract_text: Option<Field<String>>,
    pub_type_list: Option<Field<Types>>,
    license: Option<Field<String>>,
    book_or_report_details: Option<Field<Publisher>>,
}
#[derive(Deserialize)]
struct Authors {
    author: Vec<Field<Author>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Author {
    full_name: Option<Field<NativeText>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Types {
    pub_type: Vec<Field<String>>,
}
#[derive(Deserialize)]
struct Publisher {
    publisher: Option<Field<NativeText>>,
}
impl Reply for Raw {
    fn check(&self) -> Result<(), BadReply> {
        if self.result.source != "PPR" || !self.result.id.to_string().starts_with("PPR") {
            return Err(BadReply);
        }
        Ok(())
    }
}

fn record(leg: &Leg<'_>, id: &EuropePmcId) -> Result<(Record, ReplyMetadata), RequestError> {
    let endpoint = CORE
        .clone()
        .with_path(format!("/article/PPR/{id}"))
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::EuropePmc, None))?;
    let (raw, meta) = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("format", Arg::Text("json")),
                ("resultType", Arg::Text("core")),
            ]),
            biomcp_fetch::Read::<Raw>::json()
                .filter_body(&|body| crate::article_rights::europepmc(leg, body)),
        )
        .map(|received| (received.value, received.meta))?;
    if raw.result.id != *id {
        return Err(biomcp_fetch::bad_reply(
            biomcp_types::Source::EuropePmc,
            None,
        ));
    }
    Ok((raw.result, meta))
}
/// Resolve a preprint directly from its checked core metadata record.
pub fn core(
    leg: &Leg<'_>,
    id: &EuropePmcId,
) -> Result<(Mapped<Article>, ReplyMetadata), RequestError> {
    let (raw, meta) = record(leg, id)?;
    let mut p = Problems {
        source: biomcp_types::Source::EuropePmc,
        record: provenance(id),
        list: Vec::new(),
    };
    let abstract_parts = p
        .field("abstract_parts", raw.abstract_text)
        .filter(|s| !s.trim().is_empty())
        .and_then(|xml| {
            p.optional(
                "abstract_parts",
                Some(fragment(&xml).and_then(|xml| jats::markdown(&xml))),
            )
        })
        .map(|text| AbstractPart { label: None, text })
        .into_iter()
        .collect();
    let authors = p
        .field("authors", raw.author_list)
        .map(|a| a.author)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|a| {
            let a = p.field("authors", Some(a))?;
            p.source_field("authors", a.full_name)
        })
        .collect();
    let types = p
        .field("types", raw.pub_type_list)
        .map(|t| t.pub_type)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|t| p.field("types", Some(t)))
        .filter_map(|t| crate::pubmed::publication_type(&t))
        .collect();
    let publisher = p
        .field("journal", raw.book_or_report_details)
        .and_then(|r| p.source_field("journal", r.publisher));
    let value = Article {
        text_withheld: if leg.disabled_use(biomcp_types::Use::PerRecord)
            && !crate::article_rights::permitted(
                raw.license
                    .as_ref()
                    .and_then(Field::as_ref)
                    .map(|code| pmc_oa::license(code)),
                true,
            ) {
            Some(crate::article_rights::text_withheld(
                &biomcp_types::ArticleInput::EuropePmc(raw.id.clone()),
            ))
        } else {
            None
        },
        europepmc_id: Some(raw.id),
        pmid: p.field("pmid", raw.pmid),
        doi: p.field("doi", raw.doi),
        pmcid: None,
        title: p.source_field("title", raw.title),
        journal: publisher,
        date: p.field("date", raw.first_publication_date),
        electronic_date: None,
        types,
        authors,
        abstract_parts,
        corrections: Box::default(),
    };
    Ok((
        Mapped {
            refusals: crate::Refusal::None,
            value,
            problems: p.list,
        },
        meta,
    ))
}
/// Read JATS with the license owned by its core record, rather than inferring license XML.
pub fn read(
    leg: &Leg<'_>,
    id: &EuropePmcId,
) -> Result<Mapped<Option<pmc_oa::Document>>, RequestError> {
    let (raw, _) = record(leg, id)?;
    let mut p = Problems {
        source: biomcp_types::Source::EuropePmc,
        record: provenance(id),
        list: Vec::new(),
    };
    let malformed_license = matches!(raw.license, Some(Field::Invalid(_)));
    let license = p.field("license", raw.license);
    if malformed_license {
        return Ok(Mapped {
            refusals: crate::Refusal::Refused,
            value: None,
            problems: p.list,
        });
    }
    pmc_oa::permitted(
        leg,
        biomcp_types::Source::EuropePmc,
        pmc_oa::license(license.as_deref().unwrap_or("")),
    )?;
    let endpoint = FULLTEXT
        .clone()
        .with_path(format!("/{id}/fullTextXML"))
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::EuropePmc, None))?;
    let mut body = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<jats::Body>::xml(),
        )?
        .value;
    if body.title.is_none() {
        body.title = p.source_field("title", raw.title);
    }
    let value = pmc_oa::Document {
        url: Text::new(&format!("{}/{id}/fullTextXML", SPEC.base))
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::EuropePmc, None))?,
        license: pmc_oa::license(license.as_deref().unwrap_or("")),
        version: None,
        manuscript: None,
        supplements: Default::default(),
        problems: body
            .problems
            .iter()
            .cloned()
            .map(|mut p| {
                p.source = biomcp_types::Source::EuropePmc;
                p.record = provenance(id);
                p
            })
            .collect(),
        body,
    };
    p.list.extend(value.problems.clone());
    Ok(Mapped {
        refusals: crate::Refusal::None,
        value: Some(value),
        problems: p.list,
    })
}

// Europe PMC abstractText is an HTML fragment. A comparison such as p<0.001 is
// text, not a tag; escape that literal before the checked XML boundary. Broken
// tags still fail that boundary, and no markup or decoded text is discarded.
fn fragment(mut html: &str) -> Result<String, BadReply> {
    let mut xml = String::from("<abstract>");
    while let Some(start) = html.find('<') {
        xml.push_str(&html[..start]);
        html = &html[start..];
        if !html
            .as_bytes()
            .get(1)
            .is_some_and(|c| c.is_ascii_alphabetic() || matches!(c, b'/' | b'!' | b'?'))
        {
            xml.push_str("&lt;");
            html = &html[1..];
            continue;
        }
        // Consume a whole markup event, so comparisons in CDATA stay literal
        // and malformed attributes cannot be repaired into accepted markup.
        let mut reader = quick_xml::Reader::from_str(html);
        reader.config_mut().check_end_names = false;
        reader.config_mut().allow_unmatched_ends = true;
        match reader.read_event().map_err(|_| BadReply)? {
            quick_xml::events::Event::Start(tag) | quick_xml::events::Event::Empty(tag) => {
                for attribute in tag.attributes() {
                    if attribute.map_err(|_| BadReply)?.value.contains('<') {
                        return Err(BadReply);
                    }
                }
            }
            _ => {}
        }
        let end = reader.buffer_position() as usize;
        xml.push_str(&html[..end]);
        html = &html[end..];
    }
    xml.push_str(html);
    xml.push_str("</abstract>");
    Ok(xml)
}
