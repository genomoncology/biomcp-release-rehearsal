use crate::{native_text::NativeText, raw::Field};
use biodata::{PartialDate, Text, UniProtIsoform};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Entry {
    pub primary_accession: UniProtIsoform,
    pub uni_protkb_id: Option<Field<NativeText>>,
    pub entry_type: Text,
    pub organism: Organism,
    pub protein_description: Option<Field<Description>>,
    pub sequence: Option<Field<Sequence>>,
    pub genes: Option<Field<Vec<Field<Gene>>>>,
    pub entry_audit: Option<Field<Audit>>,
    pub features: Option<Field<Vec<Field<super::sections_raw::Feature>>>>,
    pub comments: Option<Field<Vec<Field<super::sections_raw::Comment>>>>,
    #[serde(rename = "uniProtKBCrossReferences")]
    pub uni_prot_kb_cross_references:
        Option<Field<Vec<Field<super::sections_raw::CrossReference>>>>,
}
impl Entry {
    pub(in crate::uniprot) fn admitted(&self, id: &UniProtIsoform) -> bool {
        self.primary_accession == *id && self.human_reviewed()
    }
    pub(in crate::uniprot) fn human_reviewed(&self) -> bool {
        self.organism.taxon_id == 9606
            && self.entry_type.as_str() == "UniProtKB reviewed (Swiss-Prot)"
    }
}
impl Reply for Entry {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Organism {
    pub taxon_id: u32,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Description {
    pub recommended_name: Option<Field<Name>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Name {
    pub full_name: Option<Field<Value>>,
}
#[derive(Debug, Deserialize)]
pub(in crate::uniprot) struct Value {
    pub value: Option<Field<NativeText>>,
}
#[derive(Debug, Deserialize)]
pub(in crate::uniprot) struct Sequence {
    pub length: Option<Field<u32>>,
    pub value: Option<Field<biodata::AminoAcids>>,
    pub crc64: Option<Field<NativeText>>,
    pub md5: Option<Field<NativeText>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Gene {
    pub gene_name: Option<Field<Value>>,
    pub synonyms: Option<Field<Vec<Field<Value>>>>,
    pub ordered_locus_names: Option<Field<Vec<Field<Value>>>>,
    pub orf_names: Option<Field<Vec<Field<Value>>>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Audit {
    pub entry_version: Option<Field<u32>>,
    pub sequence_version: Option<Field<u32>>,
    pub first_public_date: Option<Field<PartialDate>>,
    pub last_annotation_update_date: Option<Field<PartialDate>>,
    pub last_sequence_update_date: Option<Field<PartialDate>>,
}
