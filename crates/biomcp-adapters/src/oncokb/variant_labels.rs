//! Only source-stated protein context can produce a common protein statement.
use crate::oncokb::variant::MutationEffect;
use crate::oncokb::variant_map_support as map;
use crate::{mapped::Problems, raw::Field};
use biodata::{
    Attribution, Coding, OncokbOncogenicityCode as O, Proposition, Statement, Term, Text,
    VariantRef,
};
use biomcp_types::{OncokbVariantRecord as Record, Source};

pub(super) fn records(
    oncogenic: Option<Field<String>>,
    effect: Option<Field<MutationEffect>>,
    subject: Option<&VariantRef>,
    release: Option<&Text>,
    cx: &mut Problems,
) -> Vec<Record> {
    let mut out = Vec::new();
    if let Some(label) = crate::mapped::optional_text(oncogenic, "oncogenic", cx) {
        let code = match label.as_str() {
            "Oncogenic" => Some(O::Oncogenic),
            "Likely Oncogenic" => Some(O::LikelyOncogenic),
            "Likely Neutral" => Some(O::LikelyNeutral),
            "Inconclusive" => Some(O::Inconclusive),
            "Resistance" => Some(O::Resistance),
            "Unknown" => Some(O::Unknown),
            _ => {
                cx.required::<()>("oncogenic", None);
                None
            }
        };
        out.push(classification(
            subject.filter(|_| code.is_some()),
            release,
            "oncogenic",
            Term {
                label,
                codings: code.map(Coding::OncokbOncogenicity).into_iter().collect(),
            },
            Proposition::Oncogenicity,
            Box::default(),
        ));
    }
    if let Some(effect) = cx.field("mutationEffect", effect) {
        mutation(effect, subject, release, cx, &mut out);
    }
    out
}
fn mutation(
    effect: MutationEffect,
    subject: Option<&VariantRef>,
    release: Option<&Text>,
    cx: &mut Problems,
    out: &mut Vec<Record>,
) {
    let citations = cx.field("mutationEffect.citations", effect.citations);
    let (pmids, abstracts) = if let Some(citations) = citations {
        (
            crate::mapped::optional_pmids(citations.pmids, "mutationEffect.citations.pmids", cx),
            map::abstracts(
                citations.abstracts,
                "mutationEffect.citations.abstracts",
                cx,
            ),
        )
    } else {
        (Box::default(), Box::default())
    };
    if let Some(label) =
        crate::mapped::optional_text(effect.known_effect, "mutationEffect.knownEffect", cx)
    {
        out.push(classification(
            subject,
            release,
            "mutationEffect.knownEffect",
            Term {
                label,
                codings: Box::default(),
            },
            Proposition::FunctionalImpact,
            pmids,
        ));
    }
    out.extend(
        abstracts
            .into_vec()
            .into_iter()
            .map(|row| Record::Citation {
                field: "mutationEffect.citations".into(),
                text: row.text,
                link: row.link,
            }),
    );
    let description = cx.field("mutationEffect.description", effect.description);
    crate::oncokb::narrative::push(out, "mutationEffect.description", description.as_deref());
}
fn classification(
    subject: Option<&VariantRef>,
    release: Option<&Text>,
    field: &str,
    classification: Term,
    proposition: Proposition,
    citations: Box<[biodata::Pmid]>,
) -> Record {
    if let Some(subject) = subject {
        Record::Statement {
            statement: Box::new(Statement {
                method: None,
                mode_of_inheritance: None,
                criteria_not_met: Box::default(),
                status: None,
                subject: subject.clone().into(),
                proposition,
                predicate: None,
                direction: None,
                classification,
                strength: None,
                condition: None,
                therapies: Box::default(),
                allele_origin: None,
                gene_context: subject.gene().cloned(),
                criteria: Box::default(),
                citations,
                attribution: Attribution {
                    approved: None,
                    published: None,
                    source: Source::Oncokb.text(),
                    record: None,
                    version: None,
                    release: release.cloned(),
                    submitter: None,
                    evaluated: None,
                    url: None,
                },
            }),
        }
    } else {
        Record::UnmappedLabel {
            field: field.into(),
            classification,
            citations,
            statement_unmapped: true,
        }
    }
}
