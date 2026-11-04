use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum OneOrMany<T> {
    // Try the array first: T may itself accept malformed individual values.
    Many(Vec<T>),
    One(T),
}
impl<T> OneOrMany<T> {
    pub(crate) fn values(self) -> Vec<T> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values,
        }
    }
}

// Retain malformed optional fields as mapping problems without rejecting
// unrelated fields or keeping arbitrary provider data.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Field<T> {
    Value(T),
    Invalid(serde::de::IgnoredAny),
}
impl<T> Field<T> {
    pub(crate) fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Invalid(_) => None,
        }
    }
    pub(crate) fn value(self) -> Option<T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Invalid(_) => None,
        }
    }
}

/// Numeric upstream PMID checked once before entering shared response identities.
#[derive(Debug, Deserialize)]
#[serde(try_from = "u32")]
pub(crate) struct NumericPmid(pub biodata::Pmid);
impl TryFrom<u32> for NumericPmid {
    type Error = biodata::Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        biodata::Pmid::new(value)
            .map(Self)
            .map_err(|_| biodata::Error::ArticleIdInvalid)
    }
}

/// Read provider chromosome spellings before constructing the canonical shared value.
pub(crate) fn chromosome(value: &str) -> Option<biodata::Chromosome> {
    let value = value.strip_prefix("chr").unwrap_or(value);
    match value {
        "M" => "MT".parse().ok(),
        "23" => "X".parse().ok(),
        "24" => "Y".parse().ok(),
        "25" => "MT".parse().ok(),
        _ => value.parse().ok(),
    }
}
