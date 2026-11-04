//! Select exact organization sites without changing native recruitment claims.
use crate::native_text::NativeText;
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::Text;
use biomcp_types::{NciMatchedSite, NciTrial, Source};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Site {
    org_name: Option<Field<NativeText>>,
    org_city: Option<Field<NativeText>>,
    org_state_or_province: Option<Field<NativeText>>,
    recruitment_status: Option<Field<Text>>,
}
impl super::Trial {
    /// Compact facility search card and problems for malformed or unequal source sites.
    pub fn facility_card(&mut self, name: &Text) -> Mapped<NciTrial> {
        let mut card = self.card(false);
        let mut p = Problems {
            source: Source::Nci,
            record: Some(self.nci_id.clone()),
            list: Vec::new(),
        };
        let mut refusals = self.refusals;
        let expected = name.as_str().to_lowercase();
        let mut count = 0;
        for row in p.records("sites", self.sites.take(), &mut refusals) {
            let Some(org_name) = p.required_source_field("sites.org_name", row.org_name) else {
                refusals = crate::Refusal::Refused;
                continue;
            };
            if org_name.as_str().to_lowercase() != expected {
                continue;
            }
            count += 1;
            if card.matched_site.is_none() {
                card.matched_site = Some(NciMatchedSite {
                    org_name,
                    org_city: p.source_field("sites.org_city", row.org_city),
                    org_state_or_province: p
                        .source_field("sites.org_state_or_province", row.org_state_or_province),
                    recruitment_status: p.field("sites.recruitment_status", row.recruitment_status),
                });
            }
        }
        card.matching_sites = Some(count);
        if count == 0 {
            p.required::<Text>("sites.org_name", None);
        }
        Mapped {
            refusals,
            value: card,
            problems: p.list,
        }
    }
}
