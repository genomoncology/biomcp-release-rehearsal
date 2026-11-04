use crate::{
    Mapped,
    mapped::{Problems, record},
    raw::Field,
};
use biodata::{AuthorCard, AuthorHeader, AuthorId, OrcidId, Text};
use biomcp_fetch::{BadReply, Reply};
use biomcp_types::Source;
use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct Person {
    pub path: String,
    pub name: Option<Name>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct Name {
    pub path: String,
    visibility: Option<String>,
    credit_name: Option<Field<Wrapped>>,
    given_names: Option<Field<Wrapped>>,
    family_name: Option<Field<Wrapped>>,
}
#[derive(Deserialize)]
pub(super) struct Wrapped {
    #[serde(deserialize_with = "crate::native_text::deserialize")]
    pub value: biodata::Text,
}
impl Person {
    pub(super) fn card(self, id: &OrcidId) -> Result<Mapped<Option<AuthorCard>>, BadReply> {
        let Some(name) = self
            .name
            .filter(|n| n.visibility.as_deref() == Some("public"))
        else {
            return Ok(Mapped {
                refusals: crate::Refusal::None,
                value: None,
                problems: Vec::new(),
            });
        };
        let mut problems = Problems {
            source: Source::Orcid,
            record: record(&self.path),
            list: Vec::new(),
        };
        let credit = optional(&mut problems, "credit_name", name.credit_name);
        let given = optional(&mut problems, "given_names", name.given_names);
        let family = optional(&mut problems, "family_name", name.family_name);
        let display = match credit {
            Some(n) => Some(n.value),
            None => {
                let parts: Vec<_> = [given, family]
                    .into_iter()
                    .flatten()
                    .map(|n| n.value)
                    .collect();
                let value = parts.iter().map(Text::as_str).collect::<Vec<_>>().join(" ");
                if parts.is_empty() {
                    None
                } else {
                    problems.text("name", Some(&value))
                }
            }
        };
        Ok(Mapped {
            refusals: crate::Refusal::None,
            value: Some(AuthorCard {
                header: AuthorHeader {
                    id: AuthorId::Orcid(id.clone()),
                    name: display,
                    affiliations: None,
                    counts: None,
                },
                url: Some(
                    format!("https://orcid.org/{id}")
                        .parse()
                        .map_err(|_| BadReply)?,
                ),
            }),
            problems: problems.list,
        })
    }
}
impl Reply for Person {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}

pub(super) fn optional<T>(
    problems: &mut Problems,
    field: &str,
    value: Option<Field<T>>,
) -> Option<T> {
    problems.required(field, value?.value())
}
