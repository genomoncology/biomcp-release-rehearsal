//! Keep complete fragment units inside the label section allocation.
use biomcp_fetch::{InputError, RequestError};
use biomcp_types::{ReplyData, Request, Section, StepKind};
const LABEL_BYTES: usize = 8192;
pub(crate) fn bound(
    fetch: &biomcp_fetch::Fetch,
    request: &Request,
    out: &mut crate::Outcome,
    fresh: bool,
) -> Result<(), RequestError> {
    let Request::GetDrug(query) = request else {
        return Ok(());
    };
    loop {
        crate::next::finish(fetch, out);
        let ReplyData::DrugLabel {
            body: Some(body), ..
        } = &out.body
        else {
            return Ok(());
        };
        if !matches!(body.label, Section::Data { .. }) {
            return Ok(());
        }
        if serde_json::to_vec(&body.label)
            .map_err(|_| InputError::Serialization)?
            .len()
            < LABEL_BYTES
        {
            return Ok(());
        }
        let ReplyData::DrugLabel {
            body: Some(body), ..
        } = &mut out.body
        else {
            return Ok(());
        };
        let Section::Data { data, .. } = &mut body.label else {
            return Ok(());
        };
        let mut fragments = std::mem::take(&mut data.fragments).into_vec();
        if fragments.len() <= 1 {
            return Err(InputError::Serialization.into());
        }
        fragments.pop();
        let characters = fragments
            .iter()
            .map(|f| f.characters as usize)
            .sum::<usize>();
        data.text = data
            .text
            .chars()
            .take(characters)
            .collect::<String>()
            .into();
        data.fragments = fragments.into();
        let returned = data.fragments.len() as u32;
        if let Some(page) = &mut out.page {
            page.returned = returned;
        }
        out.next.retain(|step| step.0 != StepKind::Page);
        let offset = data
            .offset
            .checked_add(returned)
            .ok_or(InputError::Serialization)?;
        let request = query
            .at_section_offset(offset)
            .map_err(|_| InputError::Serialization)?;
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[biomcp_types::Source::OpenfdaLabel],
            Request::GetDrug(request),
            fresh,
        ));
    }
}
