//! PMC OA bucket listing, checked metadata and confined JATS reads.
use biomcp_fetch::{Input, Read};
mod location;
mod raw;
mod supplement;
use crate::{jats, quotas};
use biodata::License;
use biodata::{Pmcid, Pmid};

use biomcp_fetch::{Arg, Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::Source;
use std::time::Duration;
pub use supplement::{Download, Supplement, download, download_bytes};
/// Conservative application policy; no numeric provider limit is claimed.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "PMC copyright; article-specific licenses",
        url: "https://pmc.ncbi.nlm.nih.gov/about/copyright/",
        usage: biomcp_types::Use::PerRecord,
    },
    source: Source::PmcOa,
    base: "https://pmc-oa-opendata.s3.amazonaws.com",
    key: KeyUse::None,
    quota: &quotas::PMC_OA,
    deadline: Duration::from_secs(15),
};
const LIST: Endpoint = SPEC.get("/").fresh(Freshness::Days(1));
const FILE: Endpoint = SPEC.get("/").octet_stream().fresh(Freshness::Days(7));
/// Selected processing version and source text.
#[derive(Debug)]
pub struct Document {
    /// Exact public source URL used to fetch this JATS document.
    pub url: biodata::Text,
    /// License owned by the JSON metadata record.
    pub license: License,
    /// Highest numeric processing version, not publication chronology.
    pub version: Option<u32>,
    /// Author manuscript flag from metadata.
    pub manuscript: Option<bool>,
    /// Parsed source body.
    pub body: jats::Body,
    /// JATS supplements confirmed against the record media list.
    pub supplements: Vec<Supplement>,
    /// Rejected metadata and JATS fields.
    pub problems: Vec<biomcp_types::Problem>,
}

/// Return no document only when the bucket has no matching processing record.
pub fn read(
    leg: &Leg<'_>,
    id: Pmcid,
    pmid: Option<Pmid>,
) -> Result<crate::Mapped<Option<Document>>, RequestError> {
    let Some(record) = metadata(leg, id, pmid)? else {
        return Ok(crate::Mapped {
            refusals: crate::Refusal::None,
            value: None,
            problems: Vec::new(),
        });
    };
    let invalid_reply = biomcp_fetch::bad_reply(Source::PmcOa, None);
    let directory = format!("{id}.{}/", record.0);
    let (version, record) = record;
    let mut problems = crate::mapped::Problems {
        source: Source::PmcOa,
        record: crate::mapped::record(id),
        list: Vec::new(),
    };
    let license_code = problems.required_field("license", record.license_code);
    let Some(license_code) = license_code else {
        return Ok(crate::Mapped {
            refusals: crate::Refusal::Refused,
            value: None,
            problems: problems.list,
        });
    };
    permitted(leg, Source::PmcOa, license(&license_code))?;
    let loc =
        location::Location::checked(&record.xml_url, &directory).map_err(|_| invalid_reply)?;
    if !loc.path.ends_with(".xml") {
        return Err(invalid_reply);
    }
    let (url, body) = body(leg, loc)?;
    if body.pmcid.is_some_and(|found| found != id) {
        return Err(invalid_reply);
    }
    let urls = problems.fields("media_urls", record.media_urls);
    let supplements = supplement::select(&body.supplements, &urls, &directory, &mut problems)?;
    problems.list.extend(body.problems.clone());
    let manuscript = problems.field("manuscript", record.is_manuscript);
    Ok(crate::Mapped {
        refusals: body.refusals,
        value: Some(Document {
            url,
            supplements,
            license: license(&license_code),
            version: Some(version),
            manuscript,
            problems: problems.list.clone(),
            body,
        }),
        problems: Vec::new(),
    })
}
/// Read only the record-owned license; never fetch the article body.
pub fn record_license(
    leg: &Leg<'_>,
    id: Pmcid,
    pmid: Option<Pmid>,
) -> Result<Option<License>, RequestError> {
    let Some((_, record)) = metadata(leg, id, pmid)? else {
        return Ok(None);
    };
    Ok(record
        .license_code
        .and_then(crate::raw::Field::value)
        .map(|code| license(&code)))
}
fn metadata(
    leg: &Leg<'_>,
    id: Pmcid,
    pmid: Option<Pmid>,
) -> Result<Option<(u32, raw::Record)>, RequestError> {
    let invalid_reply = biomcp_fetch::bad_reply(Source::PmcOa, None);
    let prefix = format!("{id}.");
    let listing = leg
        .call(
            &LIST,
            Input::Params(&[("list-type", Arg::Num(2)), ("prefix", Arg::Text(&prefix))]),
            Read::<raw::Listing>::xml(),
        )?
        .value;
    let contents = listing.contents.as_deref().unwrap_or(&[]);
    if contents.iter().any(|e| !e.key.starts_with(&prefix)) {
        return Err(invalid_reply);
    }
    let Some(version) = contents
        .iter()
        .filter_map(|e| location::version(&e.key, id))
        .max()
    else {
        return Ok(None);
    };
    let directory = format!("{id}.{version}/");
    let path = format!("/{directory}{id}.{version}.json");
    let record = leg
        .call(
            &FILE.clone().with_path(path).map_err(|_| invalid_reply)?,
            Input::Params(&[]),
            Read::<raw::Record>::json(),
        )?
        .value;
    if record.pmcid != id || pmid.is_some_and(|expected| record.pmid != Some(expected.get())) {
        return Err(invalid_reply);
    }
    Ok(Some((version, record)))
}
pub(crate) fn license(value: &str) -> License {
    let text = value.to_ascii_uppercase().replace('-', " ");
    match text.as_str() {
        "CC BY" => License::CcBy,
        "CC BY NC" => License::CcByNc,
        "CC BY SA" => License::CcBySa,
        "CC BY NC SA" => License::CcByNcSa,
        "CC BY ND" => License::CcByNd,
        "CC BY NC ND" => License::CcByNcNd,
        "CC0" => License::Cc0,
        "PUBLIC DOMAIN" => License::PublicDomain,
        "TDM" => License::Tdm,
        _ => stated_restriction(&text),
    }
}
/// A version, URL or compact spelling must not hide a stated restriction, because
/// abstracts are withheld only for the restrictions named here. Permissive spellings
/// stay `Other`, so body text still needs an exact match above.
fn stated_restriction(text: &str) -> License {
    let words: Vec<_> = text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    let has = |names: &[&str]| words.iter().any(|word| names.contains(word));
    let compact = |suffix: &str| {
        words
            .iter()
            .any(|word| word.starts_with("CCBYNC") && word.ends_with(suffix))
    };
    if has(&["TDM"]) {
        return License::Tdm;
    }
    let noncommercial =
        has(&["NC", "NONCOMMERCIAL"]) || has(&["NON"]) && has(&["COMMERCIAL"]) || compact("");
    if !noncommercial {
        License::Other
    } else if has(&["ND", "NODERIVATIVES", "NODERIVS"]) || compact("ND") {
        License::CcByNcNd
    } else if has(&["SA", "SHAREALIKE"]) || compact("SA") {
        License::CcByNcSa
    } else {
        License::CcByNc
    }
}

fn body(
    leg: &Leg<'_>,
    loc: location::Location,
) -> Result<(biodata::Text, jats::Body), RequestError> {
    let invalid_reply = biomcp_fetch::bad_reply(Source::PmcOa, None);
    let params: Vec<_> = loc
        .md5
        .as_ref()
        .map(|v| ("md5", Arg::Text(v)))
        .into_iter()
        .collect();
    let url = biodata::Text::new(&format!(
        "{}{}{}",
        SPEC.base,
        loc.path,
        loc.md5
            .as_ref()
            .map(|v| format!("?md5={v}"))
            .unwrap_or_default()
    ))
    .map_err(|_| invalid_reply)?;
    let body = leg
        .call(
            &FILE
                .clone()
                .with_path(loc.path)
                .map_err(|_| invalid_reply)?,
            Input::Params(&params),
            Read::<jats::Body>::xml(),
        )?
        .value;
    Ok((url, body))
}

/// Complete source policy and command declaration.
pub(crate) mod decl;

mod rights;
pub use rights::{commercial, permitted};
