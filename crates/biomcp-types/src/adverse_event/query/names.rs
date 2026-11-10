//! Explicit FAERS alias lookup and a checked retained continuation identity.
use super::{AdverseEventSearch, AdverseEventSource, QueryInvalid, SearchFields, Text};

pub(super) fn validate(fields: &SearchFields) -> Result<(), QueryInvalid> {
    if fields.expand_drug_names && fields.source != AdverseEventSource::Faers {
        return Err(QueryInvalid::new(&["source", "expand_drug_names"]));
    }
    if fields.name_snapshot.is_some() && !fields.expand_drug_names {
        return Err(QueryInvalid::new(&["expand_drug_names", "name_snapshot"]));
    }
    if fields.expand_drug_names && fields.offset != 0 && fields.name_snapshot.is_none() {
        return Err(QueryInvalid::new(&["offset", "name_snapshot"]));
    }
    Ok(())
}

impl AdverseEventSearch {
    /// Explicitly requested alias lookup, never the default search behavior.
    pub fn expand_drug_names(&self) -> bool {
        self.0.expand_drug_names
    }
    /// Retained searched spellings and the resolution evidence that produced them.
    pub fn name_snapshot(&self) -> Option<&Text> {
        self.0.name_snapshot.as_ref()
    }
    /// Preserve the same name set in native offset continuations.
    pub fn with_name_snapshot(&self, snapshot: Text) -> Self {
        let mut query = self.clone();
        query.0.name_snapshot = Some(snapshot);
        query
    }
    /// Check the escaped whole native query after replacing only drug spellings.
    pub fn with_literal_drugs(&self, names: Vec<Text>) -> Result<Self, QueryInvalid> {
        let mut fields = self.0.clone();
        fields.drug = names
            .into_iter()
            .filter(|name| fields.text.as_ref().is_none_or(|text| text != name))
            .collect();
        fields.expand_drug_names = false;
        fields.name_snapshot = None;
        Self::try_from(fields)
    }
}
