use biomcp_fetch::{BadReply, KeyKind, Keys, Row};
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Debug, Deserialize)]
pub(super) struct Meta {
    pub timestamp: Option<crate::raw::Field<biodata::Text>>,
    pub total_records: usize,
}
#[derive(Debug)]
pub(super) struct Medicine {
    pub id: biodata::DrugId,
    pub source: biodata::Text,
    pub record: biodata::Text,
    pub meta: Meta,
    pub fields: BTreeMap<String, crate::raw::Field<String>>,
}
#[derive(Deserialize)]
struct Fields {
    #[serde(rename = "_bundle_meta")]
    meta: Meta,
    #[serde(flatten)]
    fields: BTreeMap<String, crate::raw::Field<String>>,
}
impl<'de> Deserialize<'de> for Medicine {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(Fields::deserialize(deserializer)?)
            .map_err(|_| serde::de::Error::custom("Invalid EMA medicine record"))
    }
}
impl TryFrom<Fields> for Medicine {
    type Error = BadReply;
    fn try_from(value: Fields) -> Result<Self, Self::Error> {
        let field = |name| match value.fields.get(name) {
            Some(crate::raw::Field::Value(value)) => Ok(value.as_str()),
            _ => Err(BadReply),
        };
        let id = field("ema_product_number")?.parse().map_err(|_| BadReply)?;
        let source = biodata::Text::new("ema").map_err(|_| BadReply)?;
        let record = biodata::Text::new(field("ema_product_number")?).map_err(|_| BadReply)?;
        Ok(Self {
            id,
            source,
            record,
            meta: value.meta,
            fields: value.fields,
        })
    }
}
pub(super) fn date(text: &str) -> Result<biodata::PartialDate, BadReply> {
    let mut parts = text.split('/');
    let (Some(day), Some(month), Some(year), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(BadReply);
    };
    format!("{year}-{month}-{day}")
        .parse()
        .map_err(|_| BadReply)
}
impl Medicine {
    pub(super) fn field(&self, name: &str) -> Option<Result<&str, BadReply>> {
        self.fields.get(name).map(|value| match value {
            crate::raw::Field::Value(value) => Ok(value.as_str()),
            crate::raw::Field::Invalid(_) => Err(BadReply),
        })
    }
}
impl Row for Medicine {
    fn check_bundle(&self, rows: usize) -> Result<(), BadReply> {
        if self.meta.total_records != rows {
            return Err(BadReply);
        }
        Ok(())
    }
    fn keys(&self) -> Keys {
        Keys {
            exact: vec![(KeyKind::Id, self.id.to_string().into())],
            words: [
                "name_of_medicine",
                "active_substance",
                "international_non_proprietary_name_common_name",
            ]
            .into_iter()
            .filter_map(|field| self.field(field).and_then(Result::ok).map(Into::into))
            .collect(),
        }
    }
}
