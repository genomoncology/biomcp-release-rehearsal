//! Each native optional value crosses the existing named-field boundary.
use super::variant as raw;
use crate::{
    mapped::{Problems, optional_text as text},
    raw::Field,
};
use biodata::{Coding, OncokbLevelCode as L, Term, Text};
use biomcp_types::{OncokbCitation, OncokbConcept};

pub(super) fn tumor(
    raw: Option<Field<raw::TumorType>>,
    field: &str,
    cx: &mut Problems,
) -> Option<OncokbConcept> {
    let raw = cx.field(field, raw)?;
    let id_field = format!("{field}.code");
    let id = text(raw.code, &id_field, cx)
        .and_then(|id| cx.optional(&id_field, Some(format!("ONCOTREE:{id}").parse())));
    let label = text(raw.name, &format!("{field}.name"), cx);
    Some(OncokbConcept { id, label })
}
pub(super) fn drugs(
    raw: Option<Field<Vec<Field<raw::Drug>>>>,
    field: &str,
    cx: &mut Problems,
) -> Box<[OncokbConcept]> {
    cx.fields(field, raw)
        .into_vec()
        .into_iter()
        .map(|raw| {
            let id_field = format!("{field}.ncitCode");
            let id = text(raw.ncit_code, &id_field, cx)
                .and_then(|id| cx.optional(&id_field, Some(format!("NCIT:{id}").parse())));
            let label = text(raw.drug_name, &format!("{field}.drugName"), cx);
            OncokbConcept { id, label }
        })
        .collect()
}
pub(super) fn abstracts(
    raw: Option<Field<Vec<Field<raw::Abstract>>>>,
    field: &str,
    cx: &mut Problems,
) -> Box<[OncokbCitation]> {
    cx.fields(field, raw)
        .into_vec()
        .into_iter()
        .map(|raw| OncokbCitation {
            text: cx
                .field(&format!("{field}.abstractContent"), raw.abstract_content)
                .map(String::into_boxed_str),
            link: cx.field(&format!("{field}.link"), raw.link),
        })
        .collect()
}
pub(super) fn level(raw: Option<Field<Text>>, field: &str, cx: &mut Problems) -> Option<Term> {
    let label = cx.field(field, raw)?;
    let code = match label.as_str() {
        "LEVEL_1" => Some(L::Level1),
        "LEVEL_2" => Some(L::Level2),
        "LEVEL_3A" => Some(L::Level3a),
        "LEVEL_3B" => Some(L::Level3b),
        "LEVEL_4" => Some(L::Level4),
        "LEVEL_R1" => Some(L::LevelR1),
        "LEVEL_R2" => Some(L::LevelR2),
        "LEVEL_Dx1" => Some(L::LevelDx1),
        "LEVEL_Dx2" => Some(L::LevelDx2),
        "LEVEL_Dx3" => Some(L::LevelDx3),
        "LEVEL_Px1" => Some(L::LevelPx1),
        "LEVEL_Px2" => Some(L::LevelPx2),
        "LEVEL_Px3" => Some(L::LevelPx3),
        _ => {
            cx.required::<()>(field, None);
            None
        }
    };
    Some(Term {
        label,
        codings: code.map(Coding::OncokbLevel).into_iter().collect(),
    })
}
