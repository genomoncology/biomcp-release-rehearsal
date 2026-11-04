use crate::{mapped::Problems, xml_element::Element};
use biodata::{
    ClinVarVariationId, ClinvarHaplotype, ClinvarHaplotypeMember, VariantId, VariantRef,
};
use biomcp_fetch::BadReply;

pub(super) fn map(
    archive: Element<'_>,
    classified: Element<'_>,
    id: VariantId,
    problems: &mut Problems,
) -> Result<Option<VariantRef>, BadReply> {
    let haplotypes = classified.children("Haplotype")?;
    if haplotypes.is_empty() && archive.attr("VariationType")?.as_deref() != Some("Haplotype") {
        return Ok(Some(VariantRef::allele(id)));
    }
    let [haplotype] = haplotypes.as_slice() else {
        problems.required::<()>("Haplotype", None);
        return Ok(None);
    };
    let member_id = variation_id(*haplotype, "Haplotype.VariationID", problems)?;
    let Some(member_id) = member_id.filter(|given| given.to_string() == id.to_string()) else {
        problems.required::<()>("Haplotype.VariationID", None);
        return Ok(None);
    };
    let accession = checked_attr(archive, "Accession", "Accession", problems)?;
    let version = checked_attr(archive, "Version", "Version", problems)?;
    let chromosomes = checked_attr(
        *haplotype,
        "NumberOfChromosomes",
        "Haplotype.NumberOfChromosomes",
        problems,
    )?;
    let mut members = Vec::new();
    for member in haplotype.children("SimpleAllele")? {
        let id = variation_id(member, "Haplotype.SimpleAllele.VariationID", problems)?;
        let allele_id = checked_attr(
            member,
            "AlleleID",
            "Haplotype.SimpleAllele.AlleleID",
            problems,
        )?;
        let (Some(id), Some(allele_id)) = (id, allele_id) else {
            return Ok(None);
        };
        members.push(ClinvarHaplotypeMember { id, allele_id });
    }
    let (Some(accession), Some(version), Some(chromosomes)) = (accession, version, chromosomes)
    else {
        return Ok(None);
    };
    let subject = ClinvarHaplotype::new(member_id, accession, version, chromosomes, members.into())
        .and_then(VariantRef::clinvar_haplotype);
    Ok(problems.optional("Haplotype", Some(subject)))
}

fn variation_id(
    element: Element<'_>,
    field: &str,
    problems: &mut Problems,
) -> Result<Option<ClinVarVariationId>, BadReply> {
    let id = problems.optional(
        field,
        element
            .attr("VariationID")?
            .map(|value| format!("clinvar:{value}").parse()),
    );
    Ok(problems.required(field, id))
}

fn checked_attr<T: std::str::FromStr>(
    element: Element<'_>,
    name: &str,
    field: &str,
    problems: &mut Problems,
) -> Result<Option<T>, BadReply> {
    let value = problems.optional(field, element.attr(name)?.map(|value| value.parse()));
    Ok(problems.required(field, value))
}
