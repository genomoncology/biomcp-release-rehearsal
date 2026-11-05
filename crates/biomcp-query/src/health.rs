//! Bounded publisher probes with independent local bundle availability.
#![deny(clippy::wildcard_enum_match_arm)]
use biodata::Text;
use biomcp_adapters::registry;
use biomcp_fetch::{Fetch, KeyUse, RequestError};
use biomcp_types::{Health, Reason, SourceHealth, SourceStatus};
use std::time::Duration;
const PROBE_BUDGET: Duration = Duration::from_secs(3);
/// Probe every implemented source with one shared command deadline and short source caps.
pub fn run(fetch: &Fetch, budget: Duration) -> Result<Health, RequestError> {
    let deadline = fetch.deadline(budget);
    let store = fetch.store_size(deadline)?;
    let sources = crate::fan_out::run(
        registry::sources()
            .map(|spec| {
                Box::new(move || {
                    let disabled = fetch.disabled(spec.source);
                    let skip = if disabled {
                        Some(Reason::Disabled)
                    } else {
                        match spec.key {
                            KeyUse::Required(c) if !fetch.has_key(c.secret) => Some(Reason::NoKey),
                            KeyUse::None | KeyUse::Optional(_) | KeyUse::Required(_) => None,
                        }
                    };
                    let leg = fetch.leg(spec, deadline.child(PROBE_BUDGET)).fresh(true);
                    let bundle = if skip.is_none() {
                        registry::bundle_state(spec, &leg)?
                    } else {
                        None
                    };
                    let mut notices = Vec::new();
                    let (status, latency_ms) = if let Some(reason) = skip {
                        (SourceStatus::Skipped(Some(reason)), None)
                    } else if fetch.is_offline() && bundle.is_some() {
                        (SourceStatus::Skipped(Some(Reason::Offline)), None)
                    } else {
                        source_probe(spec, &leg, &mut notices)?
                    };
                    let (status, latency_ms) = local_offline((status, latency_ms), bundle);
                    let mut health = SourceHealth::new(
                        spec.source,
                        Text::new(spec.terms.name)
                            .map_err(|_| biomcp_fetch::InputError::Serialization)?,
                        Text::new(spec.terms.url)
                            .map_err(|_| biomcp_fetch::InputError::Serialization)?,
                        spec.terms.usage,
                        disabled,
                        status,
                    );
                    health.bundle_size = bundle.map(|_| biomcp_types::BundleSize::SharedStorage);
                    health.notices = notices;
                    health.bundle = bundle;
                    health.components = component_health(spec.source, fetch)?.into();
                    health.latency_ms = latency_ms;
                    Ok(health)
                }) as Box<dyn FnOnce() -> Result<SourceHealth, RequestError> + Send>
            })
            .collect(),
    );
    Ok(Health {
        store,
        configuration: fetch.configuration_health().cloned(),
        disable_groups: registry::disable_groups().into(),
        sources: sources.into_iter().collect::<Result<Vec<_>, _>>()?.into(),
    })
}

fn component_health(
    source: biomcp_types::Source,
    fetch: &Fetch,
) -> Result<Vec<biomcp_types::ComponentHealth>, RequestError> {
    registry::components()
        .filter(|component| component.source == source)
        .map(|component| {
            Ok(biomcp_types::ComponentHealth {
                name: Text::new(component.name)
                    .map_err(|_| biomcp_fetch::InputError::Serialization)?,
                terms: biomcp_adapters::myvariant::components::label(component.terms)?,
                disabled: component.disabled(fetch),
            })
        })
        .collect()
}

fn local_offline(
    probe: (SourceStatus, Option<u64>),
    bundle: Option<biomcp_types::LocalBundleState>,
) -> (SourceStatus, Option<u64>) {
    use biomcp_types::LocalBundleState;
    match (biomcp_types::SourceStatusView::from(probe.0).reason, bundle) {
        (Some(Reason::Offline), Some(LocalBundleState::Fresh)) => (SourceStatus::Ok, None),
        (Some(Reason::Offline), Some(LocalBundleState::Stale)) => (SourceStatus::Stale(None), None),
        (Some(Reason::Offline), Some(LocalBundleState::Missing)) => {
            (SourceStatus::Skipped(Some(Reason::Offline)), None)
        }
        _ => probe,
    }
}

fn source_probe(
    spec: &'static biomcp_fetch::SourceSpec,
    leg: &biomcp_fetch::Leg<'_>,
    notices: &mut Vec<biomcp_types::SourceNotice>,
) -> Result<(SourceStatus, Option<u64>), RequestError> {
    let start = leg.remaining();
    let result = registry::probe(spec, leg);
    let elapsed = start.saturating_sub(leg.remaining()).as_millis() as u64;
    match result {
        Ok(()) => Ok((SourceStatus::Ok, Some(elapsed))),
        Err(error) => {
            if crate::status::local(error).is_some() {
                return Err(error);
            }
            let reason = crate::status::reason(error).unwrap_or(Reason::Unavailable);
            let retry = match error {
                RequestError::Source(error) => error.retry_after,
                RequestError::Input(_) | RequestError::Cache(_) | RequestError::Recording(_) => {
                    None
                }
            };
            notices.extend(biomcp_adapters::guidance::notices(
                spec.source,
                Some(reason),
                retry,
            ));
            let status = if reason == Reason::Disabled {
                SourceStatus::Skipped(Some(reason))
            } else {
                SourceStatus::Failed(Some(reason))
            };
            Ok((status, Some(elapsed)))
        }
    }
}
