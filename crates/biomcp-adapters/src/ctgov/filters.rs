//! Native registry advanced expressions.
use biomcp_types::TrialSearch;
use std::collections::BTreeMap;

pub(super) fn base_parameters(query: &TrialSearch) -> BTreeMap<String, String> {
    let mut fields = "NCTId,BriefTitle,OverallStatus,Phase,Condition,InterventionName,LastUpdatePostDate,HasResults".to_owned();
    if !query.country().is_empty() {
        fields.push_str(",LocationCountry");
    }
    BTreeMap::from([
        ("countTotal".into(), "true".into()),
        (
            "pageSize".into(),
            if query.count_only() {
                "1".into()
            } else {
                query.limit().to_string()
            },
        ),
        ("fields".into(), fields),
    ])
}

pub(super) fn advanced(query: &TrialSearch, broader_names: bool) -> String {
    let mut advanced = Vec::new();
    if query.exact_condition() {
        advanced.push(format!(
            "({})",
            query
                .condition()
                .iter()
                .map(|condition| format!(
                    "(AREA[Condition]EXPANSION[None]{})",
                    quote(condition.as_str())
                ))
                .collect::<Vec<_>>()
                .join(" OR ")
        ));
    }
    if let Some(id) = query.id() {
        advanced.push(format!("AREA[NCTId]{id}"));
    }
    if !query.country().is_empty() {
        advanced.push(format!(
            "({})",
            query
                .country()
                .iter()
                .map(|country| format!("AREA[LocationCountry]{}", quote(country)))
                .collect::<Vec<_>>()
                .join(" OR ")
        ));
    }
    if let Some(phase) = query.phase() {
        advanced.extend(
            phase
                .phases()
                .iter()
                .map(|phase| format!("AREA[Phase]{}", phase.as_str())),
        );
    }
    for (field, value) in [
        ("EligibilityCriteria", query.criteria()),
        (
            "LocationFacility",
            query.facility().filter(|_| !broader_names),
        ),
        (
            "LeadSponsorName",
            query.sponsor().filter(|_| !broader_names),
        ),
    ] {
        if let Some(value) = value {
            advanced.push(format!("AREA[{field}]{}", quote(value.as_str())));
        }
    }
    if let Some(study_type) = query.study_type() {
        advanced.push(format!("AREA[StudyType]{}", study_type.as_str()));
    }
    if let Some(age) = query.age() {
        advanced.push(format!(
            "(AREA[MinimumAge]RANGE[MIN,{age} years] OR AREA[MinimumAge]MISSING) AND \
            (AREA[MaximumAge]RANGE[{age} years,MAX] OR AREA[MaximumAge]MISSING)"
        ));
    }
    if query.updated_since().is_some() || query.updated_until().is_some() {
        advanced.push(format!(
            "AREA[LastUpdatePostDate]RANGE[{},{}]",
            query
                .updated_since()
                .map_or("MIN".into(), ToString::to_string),
            query
                .updated_until()
                .map_or("MAX".into(), ToString::to_string)
        ));
    }
    if query.has_results() {
        advanced.push("AREA[ResultsFirstPostDate]RANGE[MIN,MAX]".into());
    }
    advanced.join(" AND ")
}

pub(super) fn name_parameters(
    query: &TrialSearch,
    broader_names: bool,
    params: &mut BTreeMap<String, String>,
) {
    if let Some(value) = query.registry_page_token() {
        params.insert("pageToken".into(), value.into());
    }
    if broader_names {
        for (name, value) in [
            ("query.spons", query.sponsor()),
            ("query.locn", query.facility()),
        ] {
            if let Some(value) = value {
                params.insert(name.into(), value.to_string());
            }
        }
    }
}
fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
