//! Preserve the cause while decoding the tagged identity and flattened page.
use super::{GwasAssociationSection, GwasGet, GwasSectionPage, GwasStudySection, PageFields};
use crate::{QueryInvalid, Text};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Kind {
    Association,
    Study,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fields {
    kind: Kind,
    id: String,
    #[serde(default)]
    section: Option<String>,
    #[serde(default = "super::limit")]
    section_limit: u32,
    #[serde(default)]
    section_offset: u32,
    #[serde(default, deserialize_with = "crate::query::present")]
    snapshot: Option<Text>,
}
impl TryFrom<Fields> for GwasGet {
    type Error = QueryInvalid;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        let page = GwasSectionPage::try_from(PageFields {
            section_limit: fields.section_limit,
            section_offset: fields.section_offset,
            snapshot: fields.snapshot,
        })?;
        match fields.kind {
            Kind::Association => Ok(Self::Association {
                id: fields
                    .id
                    .as_str()
                    .parse()
                    .map_err(|_| QueryInvalid::new(&["id"]))?,
                section: section::<GwasAssociationSection>(fields.section)?,
                page,
            }),
            Kind::Study => Ok(Self::Study {
                id: fields
                    .id
                    .as_str()
                    .parse()
                    .map_err(|_| QueryInvalid::new(&["id"]))?,
                section: section::<GwasStudySection>(fields.section)?,
                page,
            }),
        }
    }
}
fn section<'de, T: Deserialize<'de>>(value: Option<String>) -> Result<Option<T>, QueryInvalid> {
    value
        .map(|text| {
            T::deserialize(serde::de::value::StringDeserializer::<QueryInvalid>::new(
                text.to_string(),
            ))
            .map_err(|_| QueryInvalid::new(&["id", "section"]))
        })
        .transpose()
}
impl<'de> Deserialize<'de> for GwasGet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(Fields::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
crate::query::validation::checked_query!(
    GwasGet,
    Fields,
    Some(crate::ErrorMessage::GwasGetArguments)
);
