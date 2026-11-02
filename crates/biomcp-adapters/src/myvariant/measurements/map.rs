use super::raw::{Dbnsfp, Numbers, Record, Texts};
use crate::mapped::{Problems, push_problem};
use crate::{
    Mapped,
    raw::{Field, OneOrMany},
};
use biodata::PredictorScore;
use biodata::{Attribution, Build, Text, VariantRef};

use biomcp_fetch::RequestError;
use biomcp_types::{PredictorContext, Problem, ProblemCode, Source, VariantPredictors};
struct Output {
    id: Text,
    subject: VariantRef,
    scores: Vec<PredictorScore>,
    context: Vec<PredictorContext>,
    problems: Problems,
}
pub(super) fn project(
    record: Record,
    expected: &str,
    build: Build,
    conservation: bool,
    cadd: bool,
    dbnsfp: bool,
) -> Result<Mapped<VariantPredictors>, RequestError> {
    let id = match record.id.and_then(Field::value) {
        Some(id) if id.as_str() == expected => id,
        _ => return Ok(unqualified(crate::mapped::record(expected), "_id")),
    };
    let subject = id
        .as_str()
        .split_once(':')
        .and_then(|(chromosome, suffix)| {
            let chromosome = chromosome
                .strip_prefix("chr")?
                .parse::<biodata::Chromosome>()
                .ok()?;
            let qualified = format!("{}:{suffix}", chromosome.accession(build));
            qualified.parse().ok().map(VariantRef::allele_hgvs)
        });
    let Some(subject) = subject else {
        return Ok(unqualified(Some(id), "subject_qualification"));
    };
    let mut out = Output {
        id: id.clone(),
        subject,
        scores: Vec::new(),
        context: Vec::new(),
        problems: Problems {
            source: Source::MyVariant,
            record: Some(id),
            list: Vec::new(),
        },
    };
    if cadd && let Some(cadd) = out.problems.field("cadd", record.cadd) {
        out.number("cadd.phred", cadd.phred, None)?;
        out.number("cadd.rawscore", cadd.rawscore, None)?;
        if let Some(sift) = out.problems.field("cadd.sift", cadd.sift) {
            out.number("cadd.sift.val", sift.val, sift.cat)?;
        }
    }
    if dbnsfp && let Some(mut db) = out.problems.field("dbnsfp", record.dbnsfp) {
        out.context("dbnsfp.hgvsp", db.hgvsp.take())?;
        if let Some(ensembl) = out.problems.field("dbnsfp.ensembl", db.ensembl.take()) {
            out.context("dbnsfp.ensembl.transcriptid", ensembl.transcriptid)?;
        }
        if conservation {
            project_conservation(&mut out, db)?;
        } else {
            project_prediction(&mut out, db)?;
        }
    }
    Ok(Mapped {
        refusals: crate::Refusal::None,
        value: VariantPredictors {
            subject_qualification: biomcp_types::SubjectQualification::AdapterDerivedReference,
            scores: out.scores.into(),
            components: Box::new([]),
            context: out.context.into(),
        },
        problems: out.problems.list,
    })
}
fn project_prediction(out: &mut Output, mut db: Dbnsfp) -> Result<(), RequestError> {
    if let Some(polyphen) = out.problems.field("dbnsfp.polyphen2", db.polyphen2.take()) {
        for (name, score) in [
            ("dbnsfp.polyphen2.hdiv", polyphen.hdiv),
            ("dbnsfp.polyphen2.hvar", polyphen.hvar),
        ] {
            if let Some(score) = out.problems.field(name, score) {
                out.number(&format!("{name}.score"), score.score, score.pred)?;
            }
        }
    }
    for (name, score) in [
        ("dbnsfp.revel", db.revel),
        ("dbnsfp.mutationtaster", db.mutationtaster),
    ] {
        if let Some(score) = out.problems.field(name, score) {
            out.number(&format!("{name}.score"), score.score, score.pred)?;
        }
    }
    Ok(())
}
fn project_conservation(out: &mut Output, db: Dbnsfp) -> Result<(), RequestError> {
    if let Some(gerp) = out.problems.field("dbnsfp.gerp++", db.gerp) {
        for (name, value) in [
            ("nr", gerp.nr),
            ("rs", gerp.rs),
            ("rs_rankscore", gerp.rs_rankscore),
        ] {
            out.number(&format!("dbnsfp.gerp++.{name}"), value, None)?;
        }
    }
    for (name, groups) in [("phastcons", db.phastcons), ("phylop", db.phylop)] {
        if let Some(groups) = out.problems.field(&format!("dbnsfp.{name}"), groups) {
            for (group, score) in [
                ("100way_vertebrate", groups.vertebrate),
                ("17way_primate", groups.primate),
                ("470way_mammalian", groups.mammalian),
            ] {
                if let Some(score) = out.problems.field(&format!("dbnsfp.{name}.{group}"), score) {
                    out.number(&format!("dbnsfp.{name}.{group}.score"), score.score, None)?;
                    out.number(
                        &format!("dbnsfp.{name}.{group}.rankscore"),
                        score.rankscore,
                        None,
                    )?;
                }
            }
        }
    }
    if let Some(siphy) = out.problems.field("dbnsfp.siphy_29way", db.siphy_29way) {
        out.number(
            "dbnsfp.siphy_29way.logodds_score",
            siphy.logodds_score,
            None,
        )?;
        out.number(
            "dbnsfp.siphy_29way.logodds_rankscore",
            siphy.logodds_rankscore,
            None,
        )?;
        if let Some(pi) = out.problems.field("dbnsfp.siphy_29way.pi", siphy.pi) {
            for (name, value) in [("a", pi.a), ("c", pi.c), ("g", pi.g), ("t", pi.t)] {
                out.number(&format!("dbnsfp.siphy_29way.pi.{name}"), value, None)?;
            }
        }
    }
    Ok(())
}
impl Output {
    fn context(&mut self, path: &str, values: Texts) -> Result<(), RequestError> {
        let Some(values) = self.problems.field(path, values) else {
            return Ok(());
        };
        let values: Box<[_]> = values
            .values()
            .into_iter()
            .filter_map(|raw| self.problems.field(path, Some(raw)))
            .collect();
        if !values.is_empty() {
            self.context.push(PredictorContext {
                field: Text::new(path).map_err(|_| super::super::leg_bad())?,
                values,
            });
        }
        Ok(())
    }
    fn number(&mut self, path: &str, values: Numbers, labels: Texts) -> Result<(), RequestError> {
        let label = match &labels {
            Some(Field::Value(OneOrMany::One(Field::Value(label)))) => Some(label.clone()),
            _ => None,
        };
        let label_path = path
            .strip_suffix(".val")
            .map(|prefix| format!("{prefix}.cat"))
            .or_else(|| {
                path.strip_suffix(".score")
                    .map(|prefix| format!("{prefix}.pred"))
            });
        if let Some(label_path) = label_path
            && (!matches!(&values, Some(Field::Value(OneOrMany::One(_))))
                || matches!(
                    &labels,
                    Some(Field::Value(OneOrMany::Many(_)))
                        | Some(Field::Value(OneOrMany::One(Field::Invalid(_))))
                        | Some(Field::Invalid(_))
                ))
        {
            self.context(&label_path, labels)?;
        }
        let Some(values) = values else { return Ok(()) };
        let Field::Value(values) = values else {
            push_problem(
                &mut self.problems.list,
                Source::MyVariant,
                Some(self.id.clone()),
                path,
            );
            return Ok(());
        };
        let array = matches!(&values, OneOrMany::Many(_));
        for (index, raw) in values.values().into_iter().enumerate() {
            let Some(score) = self.problems.field(path, Some(raw)) else {
                continue;
            };
            self.scores.push(PredictorScore {
                gene: None,
                transcript: None,
                subject: self.subject.clone(),
                method: Text::new(&if array {
                    format!("{path}[{index}]")
                } else {
                    path.to_owned()
                })
                .map_err(|_| super::super::leg_bad())?,
                score,
                label: if array { None } else { label.clone() },
                attribution: Attribution {
                    approved: None,
                    published: None,
                    source: Source::MyVariant.text(),
                    record: Some(self.id.clone()),
                    version: None,
                    release: None,
                    submitter: None,
                    evaluated: None,
                    url: None,
                },
            });
        }
        Ok(())
    }
}

fn unqualified(id: Option<Text>, field: &str) -> Mapped<VariantPredictors> {
    Mapped {
        refusals: crate::Refusal::Refused,
        value: VariantPredictors {
            subject_qualification: biomcp_types::SubjectQualification::AdapterDerivedReference,
            scores: Box::new([]),
            components: Box::new([]),
            context: Box::new([]),
        },
        problems: vec![Problem {
            source: Source::MyVariant,
            record: id,
            field: field.into(),
            code: ProblemCode::ValueUnknown,
        }],
    }
}
