use super::raw::Record as RawRecord;
use biodata::{Attribution, DrugIdentityLevel, DrugLabelReference, PartialDate, Text};
use biomcp_fetch::RequestError;
use biomcp_types::{DrugCard, DrugHeader, DrugSection, DrugSource};
impl RawRecord {
    fn url(&self) -> Result<Text, RequestError> {
        format!(
            "https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?setid={}",
            self.set_id
        )
        .parse()
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200)))
    }
    /// Identity comes from the SPL endpoint; no name or field-shape identity join occurs.
    pub(super) fn header(&self) -> Result<DrugHeader, RequestError> {
        self.check()
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200)))?;
        Ok(DrugHeader {
            name_resolution: None,
            matched_name: None,
            id: self.set_id.clone(),
            source: DrugSource::OpenfdaLabel,
            version: self.version.clone(),
            // Display order: brand, generic, then the native SPL set ID.
            // Product-data paragraphs remain in the original label sections.
            name: Some(
                self.openfda
                    .brand_name
                    .iter()
                    .flatten()
                    .next()
                    .or_else(|| self.openfda.generic_name.iter().flatten().next())
                    .cloned()
                    .map(Into::into)
                    .unwrap_or(self.set_id.to_string().parse().map_err(|_| {
                        biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200))
                    })?),
            ),
            identity_level: DrugIdentityLevel::SplSet,
            r#type: None,
            url: Some(self.url()?),
        })
    }
    /// Version and revision ID remain source-scoped and separate from the set ID.
    pub(super) fn reference(&self) -> Result<DrugLabelReference, RequestError> {
        self.check()
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200)))?;
        let url = self.url()?;
        Ok(DrugLabelReference {
            id: self.set_id.clone(),
            version: self.version.clone(),
            revised: self.effective_time,
            url: Some(url.clone()),
            attribution: Attribution {
                approved: None,
                published: None,
                source: "openfda-label".parse().map_err(|_| {
                    biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200))
                })?,
                record: Some(self.id.to_string().parse().map_err(|_| {
                    biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200))
                })?),
                version: self.version.clone(),
                release: None,
                submitter: None,
                evaluated: None,
                url: Some(url),
            },
        })
    }
    /// An identity-only card does not copy any clinical text or authorization facts.
    pub(super) fn card(&self, retrieved: Option<PartialDate>) -> Result<DrugCard, RequestError> {
        let alternate_names = self
            .openfda
            .brand_name
            .iter()
            .flatten()
            .chain(self.openfda.generic_name.iter().flatten())
            .cloned()
            .map(Into::into)
            .collect();
        Ok(DrugCard {
            header: self.header()?,
            alternate_names,
            cross_references: Box::default(),
            parents: Box::default(),
            products: Box::default(),
            formulations: Box::default(),
            version: self.version.clone(),
            updated: self.effective_time,
            retrieved,
            available_sections: if super::HEADINGS
                .iter()
                .any(|name| self.parts(name).is_some())
            {
                vec![DrugSection::Label].into()
            } else {
                Box::default()
            },
            next: Box::default(),
        })
    }
}

/// Accepted SPL record whose provider fields remain private to the adapter.
#[derive(Debug)]
pub struct Record(pub(super) super::raw::Record);
impl Record {
    /// Map the native identity and optional name.
    pub fn header(&self) -> Result<DrugHeader, RequestError> {
        self.0.header()
    }
    /// Map the source card and original retrieval date.
    pub fn card(&self, retrieved: Option<PartialDate>) -> Result<DrugCard, RequestError> {
        self.0.card(retrieved)
    }
    /// Whether a supported heading has native text.
    pub fn has_heading(&self, heading: &str) -> bool {
        self.0.parts(heading).is_some()
    }
    /// Complete brand equality or generic matching under the fixed single-term rule, preserving spacing.
    pub fn exact_name_match(&self, name: &biodata::Text) -> Option<biomcp_types::DrugNameMatch> {
        let exact = |names: &Option<Vec<crate::native_text::NativeText>>| {
            names
                .iter()
                .flatten()
                .any(|n| biomcp_types::DrugNameMatch::same_name(n.as_str(), name.as_str()))
        };
        let generic = self
            .0
            .openfda
            .generic_name
            .iter()
            .flatten()
            .any(|n| biomcp_types::DrugNameMatch::generic_name(n.as_str(), name.as_str()));
        biomcp_types::DrugNameMatch::from_fields(
            exact(&self.0.openfda.brand_name) && !exact(&self.0.openfda.generic_name),
            generic,
        )
    }
    /// Preserve search's existing complete-name and spacing normalization.
    pub fn name_match(&self, name: &biodata::Text) -> Option<biomcp_types::DrugNameMatch> {
        self.0.name_match(name)
    }
    /// Map bounded original label fragments.
    pub fn label(
        &self,
        heading: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Option<biomcp_types::DrugLabelText>, RequestError> {
        self.0.label(heading, offset, limit)
    }
}
