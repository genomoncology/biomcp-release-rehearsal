//! Checked protein substitutions and no-change notation; no allele inference.
//! <https://hgvs-nomenclature.org/21.1.3/recommendations/protein/substitution/>
use biodata::text_value;
use std::str::FromStr;
const AMINO: &[(&str, &str)] = &[
    ("A", "Ala"),
    ("R", "Arg"),
    ("N", "Asn"),
    ("D", "Asp"),
    ("C", "Cys"),
    ("E", "Glu"),
    ("Q", "Gln"),
    ("G", "Gly"),
    ("H", "His"),
    ("I", "Ile"),
    ("L", "Leu"),
    ("K", "Lys"),
    ("M", "Met"),
    ("F", "Phe"),
    ("P", "Pro"),
    ("S", "Ser"),
    ("T", "Thr"),
    ("W", "Trp"),
    ("Y", "Tyr"),
    ("V", "Val"),
    ("U", "Sec"),
    ("O", "Pyl"),
    ("*", "Ter"),
];
/// Explicit protein substitution or no-change notation with checked residue codes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProteinQuery {
    original: Box<str>,
    from: usize,
    position: u32,
    to: Option<usize>,
}
impl FromStr for ProteinQuery {
    type Err = super::gene::QueryInvalid;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let original = value;
        let value = value.strip_prefix("p.").unwrap_or(value);
        let first = value
            .find(|c: char| c.is_ascii_digit())
            .ok_or_else(|| super::gene::QueryInvalid::new(&["variant"]))?;
        let end = value
            .rfind(|c: char| c.is_ascii_digit())
            .ok_or_else(|| super::gene::QueryInvalid::new(&["variant"]))?
            + 1;
        let residue = |v: &str| {
            AMINO
                .iter()
                .position(|(one, three)| v == *one || v == *three)
                .ok_or_else(|| super::gene::QueryInvalid::new(&["variant"]))
        };
        let digits = value
            .get(first..end)
            .ok_or_else(|| super::gene::QueryInvalid::new(&["variant"]))?;
        if digits.starts_with('0') {
            return Err(super::gene::QueryInvalid::new(&["variant"]));
        }
        if !digits.bytes().all(|c| c.is_ascii_digit()) {
            return Err(super::gene::QueryInvalid::new(&["variant"]));
        }
        let position = digits
            .parse()
            .map_err(|_| super::gene::QueryInvalid::new(&["variant"]))?;
        let from = residue(
            value
                .get(..first)
                .ok_or_else(|| super::gene::QueryInvalid::new(&["variant"]))?,
        )?;
        let alternate = value
            .get(end..)
            .ok_or_else(|| super::gene::QueryInvalid::new(&["variant"]))?;
        let to = if alternate == "=" {
            None
        } else {
            Some(residue(alternate)?)
        };
        let original = if to.is_none() {
            let (_, three) = AMINO
                .get(from)
                .ok_or_else(|| super::gene::QueryInvalid::new(&["variant"]))?;
            format!("p.{three}{position}=").into_boxed_str()
        } else {
            original.into()
        };
        Ok(Self {
            original,
            from,
            position,
            to,
        })
    }
}
impl ProteinQuery {
    /// Whether this notation explicitly states that the residue is unchanged.
    pub(crate) fn is_synonymous(&self) -> bool {
        self.to.is_none()
    }
    /// Literal spellings searched by readers, in stable order.
    pub(crate) fn forms(&self) -> [String; 4] {
        let (from, from_three) = AMINO.get(self.from).copied().unwrap_or(("", ""));
        let (to, to_three) = self
            .to
            .and_then(|to| AMINO.get(to).copied())
            .unwrap_or(("=", "="));
        let one = format!("{from}{}{to}", self.position);
        let three = format!("{from_three}{}{to_three}", self.position);
        [
            one.clone(),
            three.clone(),
            format!("p.{one}"),
            format!("p.{three}"),
        ]
    }
}

impl std::fmt::Display for ProteinQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.original)
    }
}
impl ProteinQuery {
    /// Caller spelling, with synonymous changes written in canonical HGVS form.
    pub(crate) fn as_str(&self) -> &str {
        &self.original
    }
}
text_value!(ProteinQuery, "ProteinQuery", {
    let residues = AMINO
        .iter()
        .flat_map(|(one, three)| [regex_escape(one), (*three).to_owned()])
        .collect::<Vec<_>>()
        .join("|");
    format!(
        "^(p\\.)?({residues}){}({residues}|=)$",
        biodata::schema_pattern::decimal(u32::MAX.into())
    )
});
fn regex_escape(value: &str) -> String {
    if value == "*" {
        "\\*".into()
    } else {
        value.into()
    }
}

/// Check a consumer search substitution and return its literal source spellings.
pub fn protein_query_forms(text: &str) -> Result<[String; 4], super::gene::QueryInvalid> {
    text.parse::<ProteinQuery>().map(|p| p.forms())
}
