//! Preserve the registry cursor and the matching mode that produced it.
use super::{Fields, TrialSearch, TrialSource};
use crate::{QueryInvalid, Text};

const OWNED: &str = "biomcp:ctgov-names:";
const BROADER: &str = "biomcp:ctgov-names:v1:";

impl TrialSearch {
    /// Restart discovery with every filter retained and no registry cursor or position.
    pub fn first_page(&self) -> Self {
        let mut first = self.clone();
        first.0.next_page = None;
        first.0.offset = 0;
        first
    }

    /// A broader registry name search produced this continuation.
    pub fn broader_names(&self) -> bool {
        self.next_page()
            .is_some_and(|token| token.as_str().starts_with(BROADER))
    }

    /// Registry token alone; the private matching marker never reaches the provider.
    pub fn registry_page_token(&self) -> Option<&str> {
        self.next_page().map(|token| {
            token
                .as_str()
                .strip_prefix(BROADER)
                .unwrap_or(token.as_str())
        })
    }

    /// Wrap only a broader continuation; exact native tokens remain unchanged.
    pub fn broader_page_token(token: &Text) -> Result<Text, biodata::Error> {
        Text::new(&format!("{BROADER}{token}"))
    }
}

pub(super) fn validate(fields: &Fields) -> Result<(), QueryInvalid> {
    let Some(token) = fields.next_page.as_ref() else {
        if fields.offset != 0 {
            return Err(QueryInvalid::new(&["offset", "next_page"])
                .with_message(crate::ErrorMessage::TrialSearchCursor));
        }
        return Ok(());
    };
    if !token.as_str().starts_with(OWNED) {
        return Ok(());
    }
    let valid = token
        .as_str()
        .strip_prefix(BROADER)
        .is_some_and(|native| Text::new(native).is_ok_and(|value| value.as_str() == native));
    if !valid
        || fields.source != TrialSource::Ctgov
        || (fields.sponsor.is_none() && fields.facility.is_none())
    {
        return Err(QueryInvalid::new(&["next_page"]));
    }
    Ok(())
}
