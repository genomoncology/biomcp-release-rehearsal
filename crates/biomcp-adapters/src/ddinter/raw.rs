use biodata::Text;
use biomcp_fetch::{KeyKind, Keys, Row};
use serde::Deserialize;

/// One checked pair from the publisher's native CSV schema.
#[derive(Debug, Deserialize)]
#[serde(try_from = "Fields")]
pub(super) struct NativeRow(Fields);
#[derive(Debug, Deserialize)]
struct Fields {
    #[serde(rename = "DDInterID_A")]
    id_a: Text,
    #[serde(rename = "Drug_A")]
    name_a: Option<crate::raw::Field<crate::native_text::NativeText>>,
    #[serde(rename = "DDInterID_B")]
    id_b: Text,
    #[serde(rename = "Drug_B")]
    name_b: Option<crate::raw::Field<crate::native_text::NativeText>>,
    #[serde(rename = "Level")]
    level: Option<crate::raw::Field<Text>>,
}
impl TryFrom<Fields> for NativeRow {
    type Error = biomcp_fetch::BadReply;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        for id in [&fields.id_a, &fields.id_b] {
            if !valid_id(id.as_str()) {
                return Err(biomcp_fetch::BadReply);
            }
        }
        Ok(Self(fields))
    }
}
fn valid_id(id: &str) -> bool {
    id.strip_prefix("DDInter")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}
impl NativeRow {
    /// Published identity at the first end of the native pair.
    pub(super) fn id_a(&self) -> &str {
        self.0.id_a.as_str()
    }
    /// Published identity at the second end of the native pair.
    pub(super) fn id_b(&self) -> &str {
        self.0.id_b.as_str()
    }
    /// Native display name for the first identity.
    pub(super) fn name_a(&self) -> Option<&str> {
        self.0
            .name_a
            .as_ref()
            .and_then(crate::raw::Field::as_ref)
            .map(|v| v.as_str())
    }
    /// Native display name for the second identity.
    pub(super) fn name_b(&self) -> Option<&str> {
        self.0
            .name_b
            .as_ref()
            .and_then(crate::raw::Field::as_ref)
            .map(|v| v.as_str())
    }
    /// Rejected fields retain the native pair provenance.
    pub(super) fn problems(&self, record: Option<Text>) -> Vec<biomcp_types::Problem> {
        let mut problems = Vec::new();
        for (field, valid) in [
            ("Drug_A", self.name_a().is_some()),
            ("Drug_B", self.name_b().is_some()),
            ("Level", self.level().is_some()),
        ] {
            if !valid {
                crate::mapped::push_problem(
                    &mut problems,
                    biomcp_types::Source::Ddinter,
                    record.clone(),
                    field,
                );
            }
        }
        problems
    }
    /// Native interaction level, without interpretation.
    pub(super) fn level(&self) -> Option<&Text> {
        self.0.level.as_ref().and_then(crate::raw::Field::as_ref)
    }
}
impl Row for NativeRow {
    fn keys(&self) -> Keys {
        Keys {
            exact: vec![
                (KeyKind::Id, self.id_a().into()),
                (KeyKind::Id, self.id_b().into()),
            ],
            words: [self.name_a(), self.name_b()]
                .into_iter()
                .flatten()
                .map(Into::into)
                .collect(),
        }
    }
}
