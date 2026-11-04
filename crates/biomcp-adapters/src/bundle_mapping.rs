//! Keep rejected bundle rows beside successful synchronization values.
use crate::mapped::{Mapped, problem};
use biomcp_types::Problem;

/// Convert redacted bundle row rejections to the existing problem channel.
pub(crate) fn bundle_problems(
    source: biomcp_types::Source,
    info: &biomcp_fetch::BundleInfo,
) -> Vec<Problem> {
    rejection_problems(source, info.name, &info.rejections)
}

pub(crate) fn rejection_problems(
    source: biomcp_types::Source,
    name: &str,
    rejections: &[biomcp_fetch::BundleRejection],
) -> Vec<Problem> {
    rejections
        .iter()
        .map(|rejection| {
            let (part, reason) = rejection
                .reason
                .rsplit_once(": ")
                .map_or((None, rejection.reason.as_str()), |(part, reason)| {
                    (Some(part), reason)
                });
            let reason = match reason {
                "no usable query key" => "no_usable_query_key",
                "wrong number of columns" => "column_count",
                _ => "decode",
            };
            let location = match part {
                Some(part) => format!("{name}.{part}"),
                None => name.to_owned(),
            };
            problem(
                source,
                None,
                &format!("{location}.rows.{}.{reason}", rejection.row),
            )
        })
        .collect()
}

/// Preserve the bundle decoder's actual row refusal events.
pub(crate) fn bundle_refusals(info: &biomcp_fetch::BundleInfo) -> crate::Refusal {
    let mut refusals = crate::Refusal::None;
    for _ in &info.rejections {
        refusals = crate::Refusal::Refused;
    }
    refusals
}

/// Convert ordinary bundle synchronization metadata with its rejected rows.
pub(crate) fn bundle_sync(
    source: biomcp_types::Source,
    info: biomcp_fetch::BundleInfo,
) -> Mapped<biomcp_types::SyncResult> {
    Mapped {
        refusals: bundle_refusals(&info),
        problems: bundle_problems(source, &info),
        value: biomcp_types::SyncResult {
            native_rows: None,
            snapshot: None,
            etag: None,
            last_modified: None,
            duplicate_identities: None,
            rows: info.rows,
            bytes: info.bytes,
            downloaded_bytes: info.downloaded_bytes,
        },
    }
}
