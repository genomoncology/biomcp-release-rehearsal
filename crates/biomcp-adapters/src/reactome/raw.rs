use crate::raw::Field;
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
use std::num::NonZeroU64;

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum Search {
    Page {
        results: Vec<Group>,
        #[serde(rename = "rowCount")]
        count: u32,
        #[serde(rename = "numberOfMatches")]
        total: u64,
    },
    Empty(Empty),
}
#[derive(Deserialize)]
pub(super) struct Group {
    pub entries: Vec<Hit>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Hit {
    pub st_id: String,
    pub name: Option<Field<String>>,
    pub exact_type: String,
    pub species: Vec<String>,
}
#[derive(Deserialize)]
pub(super) struct Empty {
    code: u16,
    reason: String,
    url: String,
    pub messages: Vec<String>,
    #[serde(rename = "targets")]
    _targets: (),
}
impl Reply for Search {
    fn check(&self) -> Result<(), BadReply> {
        let valid = match self {
            Self::Page {
                results,
                count,
                total,
            } => {
                *total > 0
                    && results.iter().map(|g| g.entries.len()).sum::<usize>() == *count as usize
                    && *count <= 50
                    && u64::from(*count) <= *total
            }
            Self::Empty(v) => {
                v.code == 404
                    && v.reason == "NOT_FOUND"
                    && v.url == "http://reactome.org/ContentService/search/query"
                    && v.messages.len() == 1
            }
        };
        valid.then_some(()).ok_or(BadReply)
    }
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        match (self, status) {
            (Self::Page { .. }, 200) | (Self::Empty(_), 404) => self.check(),
            _ => Err(BadReply),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Card {
    pub st_id: String,
    pub display_name: Option<Field<String>>,
    pub species_name: String,
    pub schema_class: String,
    pub st_id_version: Option<Field<String>>,
    pub summation: Option<Field<Vec<Field<Summation>>>>,
    pub literature_reference: Option<Field<Vec<Field<Literature>>>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Summation {
    pub db_id: NonZeroU64,
    pub text: Option<Field<String>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Literature {
    pub db_id: NonZeroU64,
    pub pub_med_identifier: Option<Field<u32>>,
    pub title: Option<Field<String>>,
    #[serde(alias = "uniformResourceLocator")]
    pub url: Option<Field<String>>,
}
impl Reply for Card {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
pub(super) struct Cards(pub Vec<Card>);
impl Reply for Cards {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
pub(super) struct Version(pub u64);
impl Reply for Version {
    fn check(&self) -> Result<(), BadReply> {
        (self.0 > 0).then_some(()).ok_or(BadReply)
    }
}
