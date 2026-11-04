use biomcp_fetch::{BadReply, Reply};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};
use std::{collections::BTreeMap, fmt};

#[derive(Debug, Deserialize)]
pub(super) struct Catalog {
    #[serde(rename = "VariableFormats")]
    pub formats: Formats,
    #[serde(rename = "CancerSites")]
    pub sites: Vec<crate::raw::Field<Site>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Formats {
    pub site: Option<
        crate::raw::Field<BTreeMap<String, crate::raw::Field<crate::native_text::NativeText>>>,
    >,
}
#[derive(Debug, Deserialize)]
pub(super) struct Site {
    pub value: u32,
    pub active: Option<crate::raw::Field<bool>>,
}
impl Reply for Catalog {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Document {
    pub info: Info,
    #[serde(deserialize_with = "ordered")]
    pub data: Vec<(String, Series)>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Info {
    #[serde(rename = "y-axis")]
    pub measure: String,
    pub secondary_series: Option<String>,
    #[serde(rename = "x-axis")]
    pub axis: String,
    #[serde(rename = "key-order")]
    pub keys: Vec<String>,
    #[serde(rename = "data-fields")]
    pub fields: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Series {
    pub data_series: Vec<Vec<Option<crate::raw::Field<Cell>>>>,
    pub year_range: u32,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum Cell {
    Integer(u64),
    Number(f64),
    Text(String),
}
impl Cell {
    pub(super) fn year(&self) -> Result<u32, BadReply> {
        u32::try_from(self.integer()?).map_err(|_| BadReply)
    }

    pub(super) fn number(&self) -> Result<biodata::FiniteNumber, BadReply> {
        let value = match self {
            Self::Integer(n) => n.to_string().parse().map_err(|_| BadReply)?,
            Self::Number(n) => *n,
            Self::Text(n) => n.parse().map_err(|_| BadReply)?,
        };
        biodata::FiniteNumber::try_from(value).map_err(|_| BadReply)
    }
    pub(super) fn integer(&self) -> Result<u64, BadReply> {
        match self {
            Self::Integer(n) => Ok(*n),
            Self::Text(n) => n.parse().map_err(|_| BadReply),
            Self::Number(_) => Err(BadReply),
        }
    }
}
impl Reply for Document {
    fn check(&self) -> Result<(), BadReply> {
        let d = self;
        let (keys, fields, axis, secondary) = match d.info.measure.as_str() {
            "rate" => (
                &["sex", "race", "age_range", "stage", "subtype", "site"][..],
                &["rate", "rate_se", "rate_lower_ci", "rate_upper_ci", "count"][..],
                "stage",
                None,
            ),
            "rel_rate" => (
                &[
                    "relative_survival_interval",
                    "sex",
                    "race",
                    "age_range",
                    "site",
                ][..],
                &[
                    "year",
                    "rel_rate",
                    "rel_rate_se",
                    "rel_rate_lower_ci",
                    "rel_rate_upper_ci",
                    "modeled_rel_rate",
                    "count",
                ][..],
                "year",
                Some("modeled_rel_rate"),
            ),
            _ => return Err(BadReply),
        };
        if d.info.keys != keys
            || d.info.fields != fields
            || d.info.axis != axis
            || d.info.secondary_series.as_deref() != secondary
        {
            return Err(BadReply);
        }
        for (key, series) in &d.data {
            if key.split('_').count() != keys.len()
                || key
                    .split('_')
                    .any(|c| c.is_empty() || !c.bytes().all(|b| b.is_ascii_digit()))
                || series.data_series.iter().any(|r| r.len() != fields.len())
            {
                return Err(BadReply);
            }
        }
        Ok(())
    }
}

fn ordered<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<(String, Series)>, D::Error> {
    struct Ordered;
    impl<'de> Visitor<'de> for Ordered {
        type Value = Vec<(String, Series)>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("ordered native series")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut rows = Vec::new();
            while let Some((key, value)) = map.next_entry::<String, Series>()? {
                if rows.iter().any(|(seen, _)| seen == &key) {
                    return Err(de::Error::custom("duplicate native series"));
                }
                rows.push((key, value));
            }
            Ok(rows)
        }
    }
    de.deserialize_map(Ordered)
}
