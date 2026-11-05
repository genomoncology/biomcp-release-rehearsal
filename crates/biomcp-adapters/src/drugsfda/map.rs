//! Preserve independent native FDA products and application submission facts.
use super::raw::{Application as RawApplication, Product, Submission};
use biodata::{Attribution, AuthorizationDate, AuthorizationEvent};
use biomcp_fetch::RequestError;
use biomcp_types::{
    DrugFdaCandidate, DrugFdaDocument, DrugFdaIngredient, DrugFdaProduct, DrugFdaProperty,
    DrugFdaSubmission,
};
impl RawApplication {
    /// Expose an application name only when all native products share exact brand wording.
    pub(super) fn candidate(&self) -> DrugFdaCandidate {
        let identity = self.header();
        let name = self
            .products
            .first()
            .filter(|first| {
                self.products
                    .iter()
                    .all(|p| p.brand_name == first.brand_name)
            })
            .and_then(|product| product.brand_name.clone());
        DrugFdaCandidate {
            id: identity.id.clone(),
            name,
            identity,
            products: self.products.iter().map(Product::mapped).collect(),
        }
    }
}
impl Product {
    /// Map a native product without associating it with any submission.
    pub(super) fn mapped(&self) -> DrugFdaProduct {
        DrugFdaProduct {
            product_number: self.product_number.clone(),
            brand_name: self.brand_name.clone(),
            active_ingredients: self
                .active_ingredients
                .iter()
                .map(|i| DrugFdaIngredient {
                    name: i.name.clone(),
                    strength: i.strength.clone().map(Into::into),
                })
                .collect(),
            dosage_form: self.dosage_form.clone(),
            route: self.route.clone(),
            marketing_status: self.marketing_status.clone(),
            reference_drug: self.reference_drug.clone(),
            reference_standard: self.reference_standard.clone(),
            te_code: self.te_code.clone(),
        }
    }
}
impl Submission {
    /// Retain the application owner and native date roles without assigning product applicability.
    pub(super) fn mapped(
        &self,
        application: &RawApplication,
    ) -> Result<DrugFdaSubmission, RequestError> {
        let role = "submission_status_date"
            .parse()
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Drugsfda, Some(200)))?;
        Ok(DrugFdaSubmission {
            event: AuthorizationEvent {
                record: application.application_number.clone(),
                product_number: None,
                jurisdiction: None,
                status: self.submission_status.clone(),
                kind: self.submission_type.clone(),
                dates: self
                    .submission_status_date
                    .map(|date| AuthorizationDate { role, date })
                    .into_iter()
                    .collect(),
                indication: None,
                attribution: Attribution {
                    approved: None,
                    published: None,
                    source: "drugsfda".parse().map_err(|_| {
                        biomcp_fetch::bad_reply(biomcp_types::Source::Drugsfda, Some(200))
                    })?,
                    record: Some(application.application_number.to_string().parse().map_err(
                        |_| biomcp_fetch::bad_reply(biomcp_types::Source::Drugsfda, Some(200)),
                    )?),
                    version: None,
                    release: None,
                    submitter: application.sponsor_name.clone(),
                    evaluated: None,
                    url: None,
                },
            },
            submission_number: self.submission_number.clone(),
            supplement_purpose: None,
            review_priority: self.review_priority.clone(),
            submission_class_code: self.submission_class_code.clone(),
            submission_class_code_description: self
                .submission_class_code_description
                .clone()
                .map(Into::into),
            submission_property_type: self.submission_property_type.as_ref().map(|p| {
                p.iter()
                    .map(|p| DrugFdaProperty {
                        code: p.code.clone(),
                    })
                    .collect()
            }),
            application_docs: self
                .application_docs
                .as_ref()
                .map(|docs| {
                    docs.iter()
                        .map(|d| {
                            Ok(DrugFdaDocument {
                                id: d.id.clone(),
                                text: None,
                                url: d.url.clone(),
                                date: d.date,
                                r#type: d.r#type.clone(),
                            })
                        })
                        .collect::<Result<Box<[_]>, RequestError>>()
                })
                .transpose()?,
        })
    }
}

/// Accepted application facts with native wire fields private to the adapter.
#[derive(Debug, serde::Deserialize)]
#[serde(transparent)]
pub struct Application(pub(super) super::raw::Application);
impl Application {
    /// Exact native brand or ingredient match, original application first, then earliest approved submission.
    pub fn name_rank(&self, name: &str) -> Option<(u8, u8, Option<String>)> {
        let brand = self.0.products.iter().any(|p| {
            p.brand_name.as_ref().is_some_and(|brand| {
                biomcp_types::DrugNameMatch::same_name(brand.as_str(), name)
                    && !p.active_ingredients.iter().any(|i| {
                        i.name.as_ref().is_some_and(|ingredient| {
                            biomcp_types::DrugNameMatch::same_name(
                                brand.as_str(),
                                ingredient.as_str(),
                            )
                        })
                    })
            })
        });
        let generic = self.0.products.iter().any(|p| {
            p.active_ingredients.iter().any(|i| {
                i.name
                    .as_ref()
                    .is_some_and(|n| biomcp_types::DrugNameMatch::generic_name(n.as_str(), name))
            })
        });
        if !brand && !generic {
            return None;
        }
        let first = self
            .0
            .submissions
            .iter()
            .filter(|s| {
                s.submission_status
                    .as_ref()
                    .is_some_and(|s| s.as_str() == "AP")
            })
            .filter_map(|s| s.submission_status_date.map(|d| d.to_string()))
            .min();
        Some((
            u8::from(!brand),
            u8::from(self.0.application_number.to_string().starts_with("ANDA")),
            first,
        ))
    }
    /// Actual refused native child records.
    pub fn refusals(&self) -> crate::Refusal {
        self.0.refusals
    }
    /// Application-owned identity, without selecting a product.
    pub fn header(&self) -> biomcp_types::DrugHeader {
        self.0.header()
    }
    /// Map the independently listed native products.
    pub fn candidate(&self) -> DrugFdaCandidate {
        self.0.candidate()
    }
    /// Map product facts without associating them with submissions.
    pub fn products(&self) -> Box<[DrugFdaProduct]> {
        self.0.products.iter().map(Product::mapped).collect()
    }
    /// Explicit source-owned SPL set references.
    pub fn label_ids(&self) -> impl Iterator<Item = &biodata::DrugId> {
        self.0
            .openfda
            .as_ref()
            .and_then(|o| o.spl_set_id.as_ref())
            .into_iter()
            .flatten()
    }
    /// Native application submission count.
    pub fn submission_count(&self) -> usize {
        self.0.submissions.len()
    }
    /// Map the newest status dates first, then descending submission numbers; undated rows last.
    pub fn submissions(
        &self,
        offset: u32,
        limit: u32,
    ) -> Result<Box<[DrugFdaSubmission]>, RequestError> {
        let mut rows: Vec<_> = self.0.submissions.iter().collect();
        let date_key = |s: &Submission| {
            s.submission_status_date
                .map(|d| (d.year(), d.month(), d.day()))
        };
        rows.sort_by(|a, b| {
            date_key(b).cmp(&date_key(a)).then_with(|| {
                submission_number_key(b.submission_number.as_str())
                    .cmp(&submission_number_key(a.submission_number.as_str()))
            })
        });
        rows.into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|s| s.mapped(&self.0))
            .collect()
    }
}

// Decimal text can exceed integer widths; the original wording stays on every returned record.
fn submission_number_key(value: &str) -> (usize, &str) {
    let value = value.trim_start_matches('0');
    (value.len(), value)
}
