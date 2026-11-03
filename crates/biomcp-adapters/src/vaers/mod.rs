//! Explicit CDC WONDER summaries; no report retrieval or vaccine-name bridge.
use biomcp_fetch::Input;
mod encoding;
mod raw;
use biomcp_fetch::{Arg, BadReply, Freshness, KeyUse, Leg, SourceSpec, Terms};
use biomcp_types::{Source, Use, VaersSection, VaersSummary};
use std::time::Duration;
/// Registry-owned restricted statistical use only; no re-identification.
/// This compatibility setting applies only to CDC WONDER.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: Some("Wget/1.21.4"),
    source: Source::Vaers,
    base: "https://wonder.cdc.gov",
    terms: Terms {
        name: "CDC WONDER data use restrictions",
        url: "https://wonder.cdc.gov/datause.html",
        usage: Use::Attribution,
    },
    quota: &crate::quotas::WONDER,
    key: KeyUse::None,
    deadline: Duration::from_secs(30),
};
const TEMPLATE: &str = include_str!("request.xml");
/// Read exactly one requested table through checked shared cache admission.
pub fn read(
    leg: &Leg<'_>,
    vaccine: &str,
    section: VaersSection,
) -> Result<VaersSummary, biomcp_fetch::RequestError> {
    let xml = TEMPLATE
        .replace(
            "<name>F_D8.V14</name><value>MMR</value>",
            &format!("<name>F_D8.V14</name><value>{vaccine}</value>"),
        )
        .replace(
            "<name>B_1</name><value>D8.V13-level2</value>",
            &format!("<name>B_1</name><value>{}</value>", section.code()),
        );
    let endpoint = SPEC
        .post_form("/controller/datarequest/D8")
        .fresh(Freshness::Days(1));
    let (reply, _) = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("request_xml", Arg::Text(&xml)),
                ("accept_datause_restrictions", Arg::Text("true")),
            ]),
            biomcp_fetch::Read::<raw::Native>::xml().check(&|v| {
                if v.0
                    .vaccine
                    .get("code")
                    .is_some_and(|c| c.as_ref() == vaccine)
                    && v.0
                        .grouping
                        .get("code")
                        .is_some_and(|c| c.as_ref() == section.code())
                {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok(reply.0)
}

/// Small read-only publisher probe; never an export or synchronization.
pub(crate) const HEALTH_PATH: &str = "/datause.html";

/// Complete source policy and command declaration.
pub(crate) mod decl;
