"""Map public publisher fields to the existing BioData values, without inference."""
import csv
import json
import re

from public_dump_json import bundle_records
from public_dump_compare import VARIANT_PREFIXES, strings


def item(kind, value):
    return {"kind": kind, "value": value}


def parts(value, delimiter=","):
    return [part.strip() for part in value.split(delimiter) if part.strip()]


def concept(kind, label, identity=None):
    value = {"label": label}
    if identity:
        value["id"] = identity
    return item(kind, value)


def civic(kind, row):
    values = []
    if kind == "civic_variants":
        values += [item("identifier", "civic:" + row["variant_id"]), item("text", row["variant"])]
        for field in ("feature_name", "variant_types", "vicc_compliant_name", "iscn_name"):
            if row.get(field):
                values.append(item("text", row[field]))
        for field, value_kind in (("entrez_id", "gene_id"), ("allele_registry_id", "ca"), ("ncit_id", "ncit")):
            if row.get(field):
                values.append(item(value_kind, row[field]))
        for identity in parts(row.get("clinvar_ids", "")):
            values.append(item("clinvar_variation", "clinvar:" + identity))
        for field, prefix in (("single_variant_molecular_profile_id", "civic.profile:"), ("feature_id", "civic.feature:")):
            if row.get(field):
                values.append(item("identifier", prefix + row[field]))
        for field in ("gene", "5_prime_partner", "3_prime_partner"):
            values += [item("gene", gene) for gene in parts(row.get(field, ""))]
        values += [item("hgvs", hgvs) for hgvs in parts(row["hgvs_descriptions"])]
        for field in ("representative_transcript", "5_prime_transcript", "3_prime_transcript"):
            if row.get(field):
                values.append(item("accession", row[field]))
    elif kind == "civic_profiles":
        values.append({**item("profile", {"record": "civic:" + row["molecular_profile_id"],
                                          "label": row["name"], "members": parts(row["variant_ids"])}),
                       "mapping_reason": "profile_companion_missing"})
        values += [item("identifier", "civic:" + identity) for identity in parts(row["variant_ids"])]
        for field, prefix in (("evidence_item_ids", "civic.evidence:"), ("assertion_ids", "civic.assertion:")):
            values += [item("identifier", prefix + identity) for identity in parts(row.get(field, ""))]
        if row.get("summary"):
            values.append(item("source_text", row["summary"]))
    else:
        values.append(item("identifier", "civic.evidence:" + row["evidence_id"]))
        values.append(item("identifier", "civic:" + row["molecular_profile_id"]))
        if row.get("molecular_profile"):
            values.append(item("text", row["molecular_profile"]))
        if row.get("disease"):
            doid = row.get("doid", "")
            values.append(concept("disease", row["disease"], "DOID:" + doid if doid else None))
        for phenotype in parts(row.get("phenotypes", "")):
            values.append(concept("disease", phenotype))
        for drug in parts(row.get("therapies", "")):
            values.append(concept("drug", drug))
        values += [item("nct", identity) for identity in parts(row.get("nct_ids", ""))]
        if row.get("citation_id"):
            citation_type = {"PubMed": "citation", "DOI": "doi"}.get(row["source_type"], "text")
            values.append(item(citation_type, row["citation_id"]))
        for field in ("evidence_statement", "citation"):
            if row.get(field):
                values.append(item("source_text", row[field]))
    return values


def clinvar(row):
    values = [item("clinvar_allele", row["#AlleleID"]),
              item("clinvar_variation", "clinvar:" + row["VariationID"])]
    for gene in parts(row.get("GeneSymbol", ""), ";"):
        if gene != "-":
            values.append(item("gene", gene))
    for gene in parts(row.get("GeneID", ""), ";"):
        if gene != "-1":
            values.append(item("gene_id", gene))
    for accession in parts(row.get("RCVaccession", ""), ";"):
        if accession != "-":
            values.append(item("rcv", accession))
    rs = row.get("RS# (dbSNP)", "")
    if rs and rs != "-1":
        values.append(item("rs", "rs" + rs))
    # The preferred title may append a protein description after the transcript HGVS.
    name = row.get("Name", "")
    if ":" in name and re.search(r":[gcnrpmo]\.", name):
        values.append(item("hgvs", name.split(" ", 1)[0]))
        # Preserve unqualified protein notation as source text; never invent its accession.
        for protein in re.findall(r"\((p\.[^()]*)\)", name):
            values.append(item("text", protein))
    elif name and name != "-":
        values.append(item("text", name))
    for identity in parts(row.get("PhenotypeIDS", ""), "|"):
        if identity != "-":
            values.append(item("identifier", identity))
    for label in parts(row.get("PhenotypeList", ""), "|"):
        if label != "-":
            values.append(concept("disease", label))
    return values


def clingen(row):
    values = [item("gene", row["HGNC Gene Symbol"]),
              concept("disease", row["Disease"], row["Mondo Id"] or None)]
    if row.get("Allele Registry Id"):
        values.append(item("ca", row["Allele Registry Id"]))
    if row.get("ClinVar Variation Id"):
        values.append(item("clinvar_variation", "clinvar:" + row["ClinVar Variation Id"]))
    values += [item("hgvs", hgvs) for hgvs in re.split(r"[,;]\s*", row["HGVS Expressions"]) if hgvs]
    if row.get("Variation"):
        values.append(item("text", row["Variation"]))
    return values


REQUIRED = {
    "civic_variants": {"variant_id", "variant", "gene", "hgvs_descriptions"},
    "civic_profiles": {"molecular_profile_id", "name", "variant_ids"},
    "civic_evidence": {"evidence_id", "molecular_profile_id", "source_type", "citation_id"},
    "clinvar_summary": {"#AlleleID", "VariationID", "Name"},
    "clingen_summary": {"HGNC Gene Symbol", "Disease", "Mondo Id", "HGVS Expressions"},
}


def table_records(handle, kind):
    reader = csv.DictReader(handle, delimiter="\t")
    missing = REQUIRED[kind] - set(reader.fieldnames or [])
    if missing:
        raise ValueError("missing_columns: " + ",".join(sorted(missing)))
    for index, row in enumerate(reader, 1):
        if None in row or any(value is None for value in row.values()):
            raise ValueError(f"malformed_tsv_record: {index}")
        if kind.startswith("civic_"):
            values = civic(kind, row)
        elif kind == "clinvar_summary":
            values = clinvar(row)
        else:
            values = clingen(row)
        genes = parts(row.get("gene", "")) if kind == "civic_variants" else []
        context = None
        if kind == "civic_variants":
            context = {"id": row["variant_id"], "genes": genes}
        if kind == "clinvar_summary":
            identity = "clinvar:" + row["VariationID"]
            ids = [value["value"] for value in values
                   if isinstance(value["value"], str) and value["value"].startswith(VARIANT_PREFIXES)]
            context = ("variation/" + identity, ids, [])
        identity = "civic:" + row["molecular_profile_id"] if kind == "civic_profiles" else index
        yield identity, values, genes, None, context


SECTIONS = {
    "sequenceReference", "location", "allele", "copyNumberCount", "copyNumberChange",
    "gene", "variation", "condition", "conditionSet", "therapy", "therapyGroup", "submitter",
    "propositionPathogenicity", "propositionOncogenicity", "propositionSomaticClinicalImpact",
    "propositionClinvarCustom", "evidenceLine", "scv", "vcv", "rcv",
    # Both names appear in official release documentation; do not rename source keys.
    # https://dataexchange.clinicalgenome.org/clinvar-gkm/output-reference/overview/
    "varcond-proposition", "vartumor-proposition", "vartherapy-proposition", "varcustom-proposition",
}


def identity_value(value):
    for prefix, kind in (("clinvar:", "clinvar_variation"), ("VCV", "vcv"), ("RCV", "rcv"),
                         ("SCV", "scv"), ("rs", "rs"), ("CA", "ca"), ("ga4gh:VA.", "variant"),
                         ("MONDO:", "mondo")):
        if value.startswith(prefix):
            return item(kind, value)
    if value.startswith(("pmid:", "PMID:")):
        return item("citation", value.split(":", 1)[1])
    if ":" in value and not value.startswith(("https:", "http:", "#/")):
        return item("identifier", value)
    return None


def coding_value(record):
    """Namespace-qualified publisher codings, not inferred equivalent variants.

    https://dataexchange.clinicalgenome.org/clinvar-gkm/output-reference/cat-vrs/#mappings
    """
    code, system = record.get("code"), record.get("system")
    if not isinstance(code, str):
        return None
    known = {
        "https://www.ncbi.nlm.nih.gov/snp/": ("rs", "rs"),
        "https://www.ncbi.nlm.nih.gov/clinvar/variation/": ("clinvar_variation", "clinvar:"),
        "https://www.ncbi.nlm.nih.gov/clinvar/": ("clinvar_variation", "clinvar:"),
        "https://www.genenames.org": ("identifier", "HGNC:"),
        "https://www.omim.org/": ("identifier", "OMIM:"),
        "https://www.uniprot.org/": ("identifier", "uniprot:"),
        "https://www.ncbi.nlm.nih.gov/gene/": ("gene_id", ""),
    }
    if system in known:
        kind, prefix = known[system]
        return item(kind, code if code.startswith(prefix) else prefix + code)
    return identity_value(code)


def nested_values(record):
    """Visit only publisher identity, expression, label and citation fields."""
    if isinstance(record, list):
        for entry in record:
            yield from nested_values(entry)
    elif isinstance(record, dict):
        syntax = record.get("syntax", "")
        coding = coding_value(record)
        if coding:
            yield coding
        for key, value in record.items():
            if isinstance(value, (list, dict)):
                yield from nested_values(value)
            elif isinstance(value, str):
                if key == "value" and syntax.startswith("hgvs"):
                    yield item("hgvs", value)
                elif key == "value" and syntax:
                    yield item("expression", {"syntax": syntax, "value": value})
                elif key == "code" and coding:
                    continue
                elif key in {"id", "primaryCode", "code", "accession", "identifier", "refgetAccession"}:
                    parsed = identity_value(value)
                    if parsed:
                        yield parsed
                    elif value:
                        yield item("native_identifier" if key == "refgetAccession" else "text", value)
                elif key in {"name", "label", "title"}:
                    yield item("gene" if record.get("conceptType") == "gene" else "text", value)


def records(handle, kind):
    if kind == "clingen_evidence":
        record = json.load(handle)
        if not isinstance(record, dict) or not isinstance(record.get("evidenceLine"), list):
            raise ValueError("clingen_missing_evidence_tree")
        values = list(nested_values(record))
        condition = record.get("condition", {})
        if condition.get("label"):
            values.append(concept("disease", condition["label"], condition.get("id")))
        values.extend(criterion_citations(record))
        yield record.get("uuid", "interpretation"), values, [], None, None
        return
    if kind not in {"clinvar_gks", "clinvar_gks_record"}:
        yield from table_records(handle, kind)
        return
    for section, identity, record in gks_records(handle, kind):
        # Proposition IDs are composite native keys, not bare SCV/VCV accessions.
        if "proposition" in section.lower():
            record = {key: value for key, value in record.items() if key != "id"}
        values = list(nested_values(record))
        primary = identity_value(identity) if section in {"variation", "allele"} else None
        if primary is None:
            primary = item("identifier" if ":" in identity else "native_identifier", identity)
        if primary and record.get("id") != identity:
            values.insert(0, primary)
        if section in {"condition", "therapy"} and (record.get("name") or record.get("label")):
            values.append(concept("disease" if section == "condition" else "drug",
                                  record.get("name") or record["label"], record.get("id")))
        scalars = list(strings(record))
        ids = [value["value"] for value in values if value["kind"] in {
            "variant", "clinvar_variation", "vcv", "ca", "rs"}]
        edges = [value[2:] for value in scalars if value.startswith("#/")]
        context = (f"{section}/{identity}", ids, edges)
        yield f"{section}/{identity}", values, [], None if section in SECTIONS else section, context


def gks_records(handle, kind):
    if kind == "clinvar_gks_record":
        record = json.load(handle)
        yield "variation", record["id"], record
    else:
        yield from bundle_records(handle)


def criterion_citations(node, criterion=False):
    """Only explicit PubMed references within source criterion assessments."""
    if isinstance(node, dict):
        criterion = criterion or node.get("type") == "CriterionAssessment"
        for value in node.values():
            yield from criterion_citations(value, criterion)
    elif isinstance(node, list):
        for value in node:
            yield from criterion_citations(value, criterion)
    elif criterion and isinstance(node, str):
        for match in re.finditer(r"(?:pubmed\.ncbi\.nlm\.nih\.gov/|/pubmed/)([0-9]+)", node):
            yield item("citation", match[1])
        for match in re.finditer(r"\bPMIDs?\s*:?\s*([0-9]+(?:\s*[,;]\s*[0-9]+)*)", node, re.I):
            for pmid in re.findall(r"[0-9]+", match[1]):
                yield item("citation", pmid)
