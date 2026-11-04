use crate::native_text::NativeText;
use biodata::{GwasStudyId, Pmid, Text};
use biomcp_fetch::{BadReply, Reply};
use biomcp_types::{GwasStudy, GwasTrait};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(from = "ReceivedStudy")]
pub(in crate::gwas) struct Study {
    refusals: crate::Refusal,
    problems: Vec<biomcp_types::Problem>,
    pub accession_id: GwasStudyId,
    pub(in crate::gwas) pubmed_id: Option<u64>,
    disease_trait: Option<biodata::Text>,
    efo_traits: Vec<super::super::raw::Trait>,
    initial_sample_size: Option<NativeText>,
    replication_sample_size: Option<NativeText>,
    discovery_ancestry: Vec<NativeText>,
    replication_ancestry: Vec<NativeText>,
    genotyping_technologies: Vec<Text>,
    snp_count: Option<u64>,
    platforms: Option<NativeText>,
    array_manufacturer: Option<Vec<NativeText>>,
    gxe: Option<bool>,
    gxg: Option<bool>,
    imputed: Option<bool>,
    pooled: Option<bool>,
    user_requested: Option<bool>,
    full_summary_stats_available: Option<bool>,
    full_summary_stats: Option<Text>,
    terms_of_license: Option<Text>,
}
#[derive(Debug, Deserialize)]
pub(in crate::gwas) struct ReceivedStudy {
    pub accession_id: GwasStudyId,
    pubmed_id: Option<crate::raw::Field<u64>>,
    disease_trait: Option<crate::raw::Field<NativeText>>,
    efo_traits: Option<crate::raw::Field<Vec<crate::raw::Field<super::super::raw::Trait>>>>,
    initial_sample_size: Option<crate::raw::Field<NativeText>>,
    replication_sample_size: Option<crate::raw::Field<NativeText>>,
    discovery_ancestry: Option<crate::raw::Field<Vec<crate::raw::Field<NativeText>>>>,
    replication_ancestry: Option<crate::raw::Field<Vec<crate::raw::Field<NativeText>>>>,
    genotyping_technologies: Option<crate::raw::Field<Vec<crate::raw::Field<Text>>>>,
    snp_count: Option<crate::raw::Field<u64>>,
    platforms: Option<crate::raw::Field<NativeText>>,
    array_manufacturer: Option<crate::raw::Field<Vec<crate::raw::Field<NativeText>>>>,
    gxe: Option<crate::raw::Field<bool>>,
    gxg: Option<crate::raw::Field<bool>>,
    imputed: Option<crate::raw::Field<bool>>,
    pooled: Option<crate::raw::Field<bool>>,
    user_requested: Option<crate::raw::Field<bool>>,
    full_summary_stats_available: Option<crate::raw::Field<bool>>,
    full_summary_stats: Option<crate::raw::Field<Text>>,
    terms_of_license: Option<crate::raw::Field<Text>>,
}
impl From<ReceivedStudy> for Study {
    fn from(row: ReceivedStudy) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Gwas,
            record: crate::mapped::record(&row.accession_id),
            list: Vec::new(),
        };
        let pubmed_id = problems.field("pubmed_id", row.pubmed_id);
        let disease_trait = problems
            .required_field("disease_trait", row.disease_trait)
            .map(Into::into);
        let mut refusals = crate::Refusal::None;
        let efo_traits = problems
            .records("efo_traits", row.efo_traits, &mut refusals)
            .into_vec();
        let mut value = Self {
            refusals,
            accession_id: row.accession_id,
            pubmed_id,
            disease_trait,
            efo_traits,
            initial_sample_size: problems.field("initial_sample_size", row.initial_sample_size),
            replication_sample_size: problems
                .field("replication_sample_size", row.replication_sample_size),
            discovery_ancestry: problems
                .fields("discovery_ancestry", row.discovery_ancestry)
                .into_vec(),
            replication_ancestry: problems
                .fields("replication_ancestry", row.replication_ancestry)
                .into_vec(),
            genotyping_technologies: problems
                .fields("genotyping_technologies", row.genotyping_technologies)
                .into_vec(),
            snp_count: problems.field("snp_count", row.snp_count),
            platforms: problems.field("platforms", row.platforms),
            array_manufacturer: row
                .array_manufacturer
                .map(|v| problems.fields("array_manufacturer", Some(v)).into_vec()),
            gxe: problems.field("gxe", row.gxe),
            gxg: problems.field("gxg", row.gxg),
            imputed: problems.field("imputed", row.imputed),
            pooled: problems.field("pooled", row.pooled),
            user_requested: problems.field("user_requested", row.user_requested),
            full_summary_stats_available: problems.field(
                "full_summary_stats_available",
                row.full_summary_stats_available,
            ),
            full_summary_stats: problems.field("full_summary_stats", row.full_summary_stats),
            terms_of_license: problems.field("terms_of_license", row.terms_of_license),
            problems: Vec::new(),
        };
        for term in &mut value.efo_traits {
            for issue in term.problems.drain(..) {
                crate::mapped::push_problem(
                    &mut problems.list,
                    problems.source,
                    problems.record.clone(),
                    &format!("efo_traits.{}", issue.field),
                );
            }
        }
        value.problems = problems.list;
        value
    }
}
impl Reply for Study {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
impl Study {
    pub(in crate::gwas) fn card(self) -> crate::Mapped<GwasStudy> {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Gwas,
            record: crate::mapped::record(&self.accession_id),
            list: self.problems,
        };
        let publication = problems.optional(
            "pubmed_id",
            self.pubmed_id.map(|n| n.to_string().parse::<Pmid>()),
        );
        let value = GwasStudy {
            id: self.accession_id,
            publication,
            reported_traits: self.disease_trait.into_iter().collect(),
            ontology_traits: self
                .efo_traits
                .into_iter()
                .map(|t| GwasTrait::new(t.efo_id, t.efo_trait))
                .collect(),
            initial_sample_description: self.initial_sample_size.map(Into::into),
            replication_sample_description: self.replication_sample_size.map(Into::into),
            discovery_ancestries: self
                .discovery_ancestry
                .into_iter()
                .map(Into::into)
                .collect(),
            replication_ancestries: self
                .replication_ancestry
                .into_iter()
                .map(Into::into)
                .collect(),
            genotyping_technologies: self.genotyping_technologies.into(),
            snp_count: self.snp_count,
            platforms: self.platforms.map(Into::into),
            array_manufacturers: self
                .array_manufacturer
                .map(|values| values.into_iter().map(Into::into).collect()),
            gxe: self.gxe,
            gxg: self.gxg,
            imputed: self.imputed,
            pooled: self.pooled,
            user_requested: self.user_requested,
            summary_statistics_available: self.full_summary_stats_available,
            summary_statistics_url: self.full_summary_stats,
            license_url: self.terms_of_license,
        };
        crate::Mapped {
            refusals: self.refusals,
            value,
            problems: problems.list,
        }
    }
}
