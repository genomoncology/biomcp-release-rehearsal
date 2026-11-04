use super::raw::Record;
use crate::text_parts::{escaped_bytes, split};
use biomcp_fetch::RequestError;
use biomcp_types::{DrugLabelFragment, DrugLabelText};
// Leave the rest of the bounded label envelope available for identity and provenance.
const PAYLOAD_BYTES: usize = 6144;
struct Fragment {
    meta: DrugLabelFragment,
    text: String,
}
impl Record {
    /// Page bounded text fragments, defaulting in the consumer to one fragment.
    /// The payload budget may return fewer fragments than the supplied maximum.
    /// Offsets and totals count fragments, never characters, sentences or source array elements.
    pub(super) fn label(
        &self,
        heading: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Option<DrugLabelText>, RequestError> {
        if !(1..=50).contains(&limit) {
            return Err(biomcp_fetch::bad_reply(
                biomcp_types::Source::OpenfdaLabel,
                Some(200),
            ));
        }
        let Some(parts) = self.parts(heading) else {
            return Ok(None);
        };
        let all = fragments(parts)?;
        let total = all.len() as u64;
        let mut text = String::new();
        let mut selected = Vec::new();
        let mut bytes = 0;
        for fragment in all.into_iter().skip(offset as usize).take(limit as usize) {
            let size = escaped_bytes(&fragment.text);
            if !selected.is_empty() && bytes + size > PAYLOAD_BYTES {
                break;
            }
            bytes += size;
            text.push_str(&fragment.text);
            selected.push(fragment.meta);
        }
        Ok(Some(DrugLabelText {
            label: self.reference()?,
            heading: heading.parse().map_err(|_| {
                biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200))
            })?,
            text: text.into(),
            retrieved: None,
            fragments: selected.into(),
            offset,
            total: Some(total),
        }))
    }
}
fn fragments(parts: &[String]) -> Result<Vec<Fragment>, RequestError> {
    let mut fragments = Vec::new();
    for (source_part, part) in parts.iter().enumerate() {
        let mut source_offset = 0u32;
        let pieces = if part.is_empty() {
            vec![String::new()]
        } else {
            split(part, PAYLOAD_BYTES)
        };
        for text in pieces {
            let characters = u32::try_from(text.chars().count()).map_err(|_| {
                biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200))
            })?;
            fragments.push(Fragment {
                meta: DrugLabelFragment {
                    source_part: u32::try_from(source_part).map_err(|_| {
                        biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200))
                    })?,
                    source_offset,
                    characters,
                },
                text,
            });
            source_offset = source_offset.checked_add(characters).ok_or_else(|| {
                biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200))
            })?;
        }
    }
    Ok(fragments)
}
