use crate::{Error, text_value};
use std::{borrow::Cow, fmt, str::FromStr};
/// Published rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS,
/// chromosome genomic HGVS, or single-base chromosome-position-reference-alternate coordinates.
/// Multi-base coordinates are unsupported: use reference-qualified HGVS and
/// variant normalize --source car --hgvs <HGVS> to validate source forms.
/// Ambiguous candidates remain separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariantInput {
    text: crate::Text,
    kind: InputKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum InputKind {
    Id(crate::VariantId),
    Hgvs(crate::HgvsText),
    Chromosome,
    Coordinates(String),
}
/// Namespace selected solely from the published identifier form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantNamespace {
    /// dbSNP reference SNP identifier.
    Rs,
    /// ClinGen canonical allele identifier.
    Ca,
    /// ClinVar numeric Variation ID.
    ClinVar,
    /// Reference-qualified opaque HGVS description.
    Hgvs,
    /// Chromosome HGVS description requiring an explicit build.
    Chromosome,
    /// Identifier vocabulary is recognized but this get slice has no source route.
    Unsupported,
}
impl VariantInput {
    pub(crate) const COORDINATE_PATTERN: &'static str =
        r"^(chr)?([1-9]|1[0-9]|2[0-2]|X|Y|M|MT)-[1-9][0-9]*-[ACGT]-[ACGT]$";
    pub(crate) const CHROMOSOME_PATTERN: &'static str =
        r"^chr([1-9]|1[0-9]|2[0-2]|X|Y|M|MT):g\.[\x21-\x7e]*[0-9][\x21-\x7e]*$";

    /// Maximum bytes in one published identifier.
    pub const MAX_BYTES: usize = 2048;
    /// Check the identifier spelling and canonicalize the ClinVar namespace prefix.
    pub fn new(value: &str) -> Result<Self, Error> {
        let value = if value
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("clinvar:"))
        {
            Cow::Owned(format!("clinvar:{}", &value[8..]))
        } else {
            Cow::Borrowed(value)
        };
        let text = input_text(value.as_ref())?;
        let kind = if let Ok(id) = value.parse::<crate::VariantId>() {
            InputKind::Id(id)
        } else if positive(&value) && value.parse::<u64>().is_ok() {
            InputKind::Id(
                format!("clinvar:{value}")
                    .parse()
                    .map_err(|_| Error::VariantInputInvalid)?,
            )
        } else if let Some(hgvs) = coordinates(value.as_ref()) {
            InputKind::Coordinates(hgvs)
        } else if let Ok(hgvs) = value.parse::<crate::HgvsText>() {
            InputKind::Hgvs(hgvs)
        } else if crate::Expression::new(crate::ExpressionSyntax::HgvsG, text.clone(), None).is_ok()
        {
            InputKind::Chromosome
        } else {
            return Err(Error::VariantInputInvalid);
        };
        Ok(Self { text, kind })
    }
    /// Preserve a parsed candidate identity and apply the input text bounds once.
    pub fn from_identity(id: crate::VariantId) -> Result<Self, Error> {
        let text = input_text(&id.to_string())?;
        Ok(Self {
            text,
            kind: InputKind::Id(id),
        })
    }
    /// Coordinates use GRCh38 unless the caller supplies another assembly.
    pub fn coordinate_form(&self) -> bool {
        matches!(self.kind, InputKind::Coordinates(_))
    }
    /// Source lookup identity; VCV archive versions do not change the Variation ID.
    pub fn source_id(&self) -> Cow<'_, str> {
        if let Some(crate::VariantId::Vcv(id)) = self.identity() {
            let value = id.to_string();
            let digits = value
                .strip_prefix("VCV")
                .unwrap_or_default()
                .split('.')
                .next()
                .unwrap_or_default();
            return Cow::Owned(format!("clinvar:{}", digits.trim_start_matches('0')));
        }
        if let Some(id @ crate::VariantId::ClinVarVariation(_)) = self.identity() {
            return Cow::Owned(id.to_string());
        }
        self.native_hgvs()
    }
    /// Return the checked identifier with a canonical ClinVar prefix.
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
    /// Borrow the already checked input text without reparsing or trimming it.
    pub fn text(&self) -> &crate::Text {
        &self.text
    }
    /// Checked parenthetical gene and its exact reference accession, when supplied.
    pub fn reference_gene(&self) -> Option<(&str, &str)> {
        let (reference, _) = self.as_str().split_once(':')?;
        let (accession, gene) = reference.split_once('(')?;
        let gene = gene.strip_suffix(')')?;
        if gene.parse::<biodata::Accession>().is_ok() || gene.parse::<crate::GeneSymbol>().is_err()
        {
            return None;
        }
        Some((accession, gene))
    }
    /// Native resolver input omits a checked parenthetical gene label.
    pub fn native_hgvs(&self) -> Cow<'_, str> {
        if let InputKind::Coordinates(hgvs) = &self.kind {
            return Cow::Borrowed(hgvs);
        }
        if let Some(body) = self.as_str().strip_prefix("chrM:") {
            return Cow::Owned(format!("chrMT:{body}"));
        }
        if let Some((accession, _)) = self.reference_gene()
            && let Some((_, body)) = self.as_str().split_once(':')
        {
            Cow::Owned(format!("{accession}:{body}"))
        } else {
            Cow::Borrowed(self.as_str())
        }
    }
    /// Borrow the checked reference-qualified notation.
    pub fn hgvs(&self) -> Option<&crate::HgvsText> {
        match &self.kind {
            InputKind::Hgvs(value) => Some(value),
            _ => None,
        }
    }
    /// Retain the source identity parsed at the input boundary.
    pub fn identity(&self) -> Option<&crate::VariantId> {
        match &self.kind {
            InputKind::Id(id) => Some(id),
            _ => None,
        }
    }
    /// Classify a published namespace, without parsing molecular meaning.
    pub fn namespace(&self) -> VariantNamespace {
        match &self.kind {
            InputKind::Id(crate::VariantId::Rs(_)) => VariantNamespace::Rs,
            InputKind::Id(crate::VariantId::Ca(_)) => VariantNamespace::Ca,
            InputKind::Id(crate::VariantId::ClinVarVariation(_) | crate::VariantId::Vcv(_)) => {
                VariantNamespace::ClinVar
            }
            InputKind::Id(_) => VariantNamespace::Unsupported,
            InputKind::Hgvs(_) => VariantNamespace::Hgvs,
            InputKind::Chromosome | InputKind::Coordinates(_) => VariantNamespace::Chromosome,
        }
    }
}
fn coordinates(value: &str) -> Option<String> {
    let mut parts = value.split('-');
    let chromosome = parts
        .next()?
        .strip_prefix("chr")
        .unwrap_or(value.split('-').next()?);
    let chromosome = if chromosome == "M" { "MT" } else { chromosome };
    let position = parts.next()?;
    let reference = parts.next()?;
    let alternate = parts.next()?;
    if parts.next().is_some()
        || position.starts_with('0')
        || position.parse::<u64>().ok()? == 0
        || reference.len() != 1
        || alternate.len() != 1
        || !reference
            .bytes()
            .chain(alternate.bytes())
            .all(|b| b"ACGT".contains(&b))
    {
        return None;
    }
    // Coordinate spelling is converted to the same checked genomic expression used elsewhere.
    let expression = format!("chr{chromosome}:g.{position}{reference}>{alternate}");
    crate::Expression::new(
        crate::ExpressionSyntax::HgvsG,
        crate::Text::new(&expression).ok()?,
        None,
    )
    .ok()?;
    Some(expression)
}
fn input_text(value: &str) -> Result<crate::Text, Error> {
    if value.is_empty()
        || value.len() > VariantInput::MAX_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(Error::VariantInputInvalid);
    }
    crate::Text::new(value).map_err(|_| Error::VariantInputInvalid)
}
fn positive(value: &str) -> bool {
    !value.is_empty() && !value.starts_with('0') && value.bytes().all(|b| b.is_ascii_digit())
}
impl FromStr for VariantInput {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Error> {
        Self::new(value)
    }
}
impl fmt::Display for VariantInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
text_value!(
    VariantInput,
    "VariantInput",
    format!(
        "^(?=[\\x21-\\x7e]{{1,{}}}$)({}|{}|{}|{}|{})",
        Self::MAX_BYTES,
        crate::VariantId::pattern().replace("clinvar:", "[cC][lL][iI][nN][vV][aA][rR]:"),
        crate::HgvsText::pattern(),
        biodata::ClinVarVariationId::pattern().replace("clinvar:", ""),
        Self::CHROMOSOME_PATTERN,
        Self::COORDINATE_PATTERN
    ),
    "Accepted forms: rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS, chromosome genomic HGVS, \
    or single-base chromosome-position-reference-alternate coordinates. Multi-base coordinates are unsupported; \
    use reference-qualified HGVS and variant normalize --source car --hgvs <HGVS> to validate source forms."
);
