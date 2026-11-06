//! Borrow only requested protocol text for literal gene-context observations.
use super::{Field, Protocol, RawPage};
use crate::ctgov::{GeneContext, gene};

impl RawPage {
    pub(in crate::ctgov) fn search_contexts(
        &self,
        conditions: &[biodata::Text],
        symbol: Option<&str>,
    ) -> (bool, Vec<GeneContext>) {
        let mut mismatch = false;
        let mut contexts = Vec::new();
        for row in self.studies.iter().flatten().filter_map(Field::as_ref) {
            mismatch |= conditions.iter().any(|condition| {
                !row.protocol_section
                    .contains_condition_words(condition.as_str())
            });
            if let Some(symbol) = symbol
                && let Some(context) = row.protocol_section.gene_context(symbol)
            {
                contexts.push(context);
            }
        }
        (mismatch, contexts)
    }
}

impl Protocol {
    pub(in crate::ctgov) fn gene_context(&self, symbol: &str) -> Option<GeneContext> {
        let fields = self.gene_fields();
        let eligibility = self
            .eligibility_module
            .as_ref()
            .and_then(Field::as_ref)
            .and_then(|e| e.eligibility_criteria.as_ref())
            .and_then(Field::as_ref);
        gene::classify(symbol, &fields, eligibility.map(AsRef::as_ref)).map(|mention| GeneContext {
            nct_id: self.identification_module.nct_id,
            mention,
        })
    }

    fn gene_fields(&self) -> Vec<(&'static str, &str)> {
        let mut fields = Vec::new();
        if let Some(title) = self
            .identification_module
            .brief_title
            .as_ref()
            .and_then(Field::as_ref)
        {
            fields.push(("title", title.as_str()));
        }
        if let Some(description) = self.description_module.as_ref().and_then(Field::as_ref) {
            for (name, text) in [
                ("brief summary", &description.brief_summary),
                ("detailed description", &description.detailed_description),
            ] {
                if let Some(text) = text.as_ref().and_then(Field::as_ref) {
                    fields.push((name, text.as_ref()));
                }
            }
        }
        if let Some(conditions) = self.conditions_module.as_ref().and_then(Field::as_ref) {
            fields.extend(
                values(conditions.conditions.as_ref()).map(|text| ("condition", text.as_str())),
            );
            fields.extend(
                values(conditions.keywords.as_ref()).map(|text| ("keyword", text.as_ref())),
            );
        }
        if let Some(interventions) = self
            .arms_interventions_module
            .as_ref()
            .and_then(Field::as_ref)
        {
            fields.extend(
                values(interventions.interventions.as_ref()).filter_map(|row| {
                    row.name
                        .as_ref()
                        .and_then(Field::as_ref)
                        .map(|name| ("intervention", name.as_str()))
                }),
            );
        }
        fields
    }
}

fn values<T>(field: Option<&Field<Vec<Field<T>>>>) -> impl Iterator<Item = &T> {
    field
        .and_then(Field::as_ref)
        .into_iter()
        .flatten()
        .filter_map(Field::as_ref)
}
