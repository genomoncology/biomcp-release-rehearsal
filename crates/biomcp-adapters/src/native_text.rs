//! Admit native human text without changing identity and protocol parsing.
use biodata::Text;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
/// Native source prose admitted through the shared checked text constructor.
pub struct NativeText(Text);
impl<'de> Deserialize<'de> for NativeText {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = String::deserialize(d)?;
        Text::from_source(&value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
impl std::ops::Deref for NativeText {
    type Target = Text;
    fn deref(&self) -> &Text {
        &self.0
    }
}
impl From<NativeText> for Text {
    fn from(value: NativeText) -> Self {
        value.0
    }
}
impl std::fmt::Display for NativeText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub(crate) fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Text, D::Error> {
    NativeText::deserialize(d).map(Into::into)
}
