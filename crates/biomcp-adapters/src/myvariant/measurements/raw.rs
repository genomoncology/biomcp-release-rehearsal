use crate::raw::{Field, OneOrMany};
use biodata::{FiniteNumber, Text};

use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
pub(super) type Numbers = Option<Field<OneOrMany<Field<FiniteNumber>>>>;
pub(super) type Texts = Option<Field<OneOrMany<Field<Text>>>>;
#[derive(Deserialize)]
pub(super) struct Record {
    #[serde(rename = "_id")]
    pub id: Option<Field<Text>>,
    pub cadd: Option<Field<Cadd>>,
    pub dbnsfp: Option<Field<Dbnsfp>>,
}
impl Reply for Record {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
pub(super) struct Cadd {
    pub phred: Numbers,
    pub rawscore: Numbers,
    pub sift: Option<Field<Sift>>,
}
#[derive(Deserialize)]
pub(super) struct Sift {
    pub val: Numbers,
    pub cat: Texts,
}
#[derive(Deserialize)]
pub(super) struct Dbnsfp {
    pub hgvsp: Texts,
    pub ensembl: Option<Field<Ensembl>>,
    pub polyphen2: Option<Field<Polyphen>>,
    pub revel: Option<Field<Score>>,
    pub mutationtaster: Option<Field<Score>>,
    #[serde(rename = "gerp++")]
    pub gerp: Option<Field<Gerp>>,
    pub phastcons: Option<Field<Groups>>,
    pub phylop: Option<Field<Groups>>,
    pub siphy_29way: Option<Field<Siphy>>,
}
#[derive(Deserialize)]
pub(super) struct Ensembl {
    pub transcriptid: Texts,
}
#[derive(Deserialize)]
pub(super) struct Polyphen {
    pub hdiv: Option<Field<Score>>,
    pub hvar: Option<Field<Score>>,
}
#[derive(Deserialize)]
pub(super) struct Score {
    pub score: Numbers,
    pub rankscore: Numbers,
    pub pred: Texts,
}
#[derive(Deserialize)]
pub(super) struct Gerp {
    pub nr: Numbers,
    pub rs: Numbers,
    pub rs_rankscore: Numbers,
}
#[derive(Deserialize)]
pub(super) struct Groups {
    #[serde(rename = "100way_vertebrate")]
    pub vertebrate: Option<Field<Score>>,
    #[serde(rename = "17way_primate")]
    pub primate: Option<Field<Score>>,
    #[serde(rename = "470way_mammalian")]
    pub mammalian: Option<Field<Score>>,
}
#[derive(Deserialize)]
pub(super) struct Siphy {
    pub logodds_score: Numbers,
    pub logodds_rankscore: Numbers,
    pub pi: Option<Field<Pi>>,
}
#[derive(Deserialize)]
pub(super) struct Pi {
    pub a: Numbers,
    pub c: Numbers,
    pub g: Numbers,
    pub t: Numbers,
}
