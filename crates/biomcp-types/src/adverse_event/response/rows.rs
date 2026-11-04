use serde::{Deserialize, Serialize};
/// Lengths of present native harmonized lists; missing stays absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventIdentityCounts {
    /// Actual native unii list length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unii: Option<u32>,
    /// Actual native rxcui list length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rxcui: Option<u32>,
    /// Actual native product_ndc list length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_ndc: Option<u32>,
    /// Actual native package_ndc list length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_ndc: Option<u32>,
    /// Actual native spl_id list length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spl_id: Option<u32>,
    /// Actual native spl_set_id list length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spl_set_id: Option<u32>,
    /// Actual native application_number list length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_number: Option<u32>,
}
/// One absolute native drug row without its unbounded identifier arrays.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventDrug {
    /// Absolute native array index.
    pub index: u32,
    /// Native medicinalproduct string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medicinalproduct: Option<Box<str>>,
    /// Native role string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Box<str>>,
    /// Native indication string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indication: Option<Box<str>>,
    /// Native drugadministrationroute string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugadministrationroute: Option<Box<str>>,
    /// Native drugauthorizationnumb string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugauthorizationnumb: Option<Box<str>>,
    /// Native drugbatchnumb string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugbatchnumb: Option<Box<str>>,
    /// Native drugcumulativedosagenumb string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugcumulativedosagenumb: Option<Box<str>>,
    /// Native drugcumulativedosageunit string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugcumulativedosageunit: Option<Box<str>>,
    /// Native drugdosageform string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugdosageform: Option<Box<str>>,
    /// Native drugdosagetext string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugdosagetext: Option<Box<str>>,
    /// Native drugenddate string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugenddate: Option<Box<str>>,
    /// Native drugenddateformat string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugenddateformat: Option<Box<str>>,
    /// Native drugintervaldosagedefinition string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugintervaldosagedefinition: Option<Box<str>>,
    /// Native drugintervaldosageunitnumb string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugintervaldosageunitnumb: Option<Box<str>>,
    /// Native drugrecurreadministration string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugrecurreadministration: Option<Box<str>>,
    /// Native drugseparatedosagenumb string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugseparatedosagenumb: Option<Box<str>>,
    /// Native drugstartdate string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugstartdate: Option<Box<str>>,
    /// Native drugstartdateformat string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugstartdateformat: Option<Box<str>>,
    /// Native drugstructuredosagenumb string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugstructuredosagenumb: Option<Box<str>>,
    /// Native drugstructuredosageunit string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugstructuredosageunit: Option<Box<str>>,
    /// Native drugtreatmentduration string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugtreatmentduration: Option<Box<str>>,
    /// Native drugtreatmentdurationunit string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugtreatmentdurationunit: Option<Box<str>>,
    /// Native actiondrug string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actiondrug: Option<Box<str>>,
    /// Native drugadditional string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drugadditional: Option<Box<str>>,
    /// Sizes only; original lists live behind named pages.
    pub identity_counts: AdverseEventIdentityCounts,
}
/// Independent native reaction row without drug attribution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventReaction {
    /// Absolute native reaction index.
    pub index: u32,
    /// Source term label without invented CURIE.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub term: Option<biodata::Concept>,
    /// Native vocabulary_version string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vocabulary_version: Option<Box<str>>,
    /// Native outcome string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Box<str>>,
}
/// Native device fields; administrative addresses are excluded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventDevice {
    /// Absolute native device index.
    pub index: u32,
    /// Native sequence string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<Box<str>>,
    /// Native brand_name string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<Box<str>>,
    /// Native generic_name string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generic_name: Option<Box<str>>,
    /// Native manufacturer string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<Box<str>>,
    /// Native product_code string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_code: Option<Box<str>>,
    /// Native model string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<Box<str>>,
    /// Native catalog string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog: Option<Box<str>>,
    /// Native udi_di string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub udi_di: Option<Box<str>>,
    /// Native udi_public string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub udi_public: Option<Box<str>>,
    /// Native lot_number string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lot_number: Option<Box<str>>,
    /// Native device_operator string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_operator: Option<Box<str>>,
    /// Native implant_flag string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implant_flag: Option<Box<str>>,
    /// Native implant_date_year string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implant_date_year: Option<Box<str>>,
    /// Native date_removed_flag string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_removed_flag: Option<Box<str>>,
    /// Native date_removed_year string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_removed_year: Option<Box<str>>,
    /// Native device_age_text string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_age_text: Option<Box<str>>,
    /// Native expiration_date_of_device string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_date_of_device: Option<Box<str>>,
    /// Native device_availability string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_availability: Option<Box<str>>,
    /// Native device_evaluated_by_manufacturer string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_evaluated_by_manufacturer: Option<Box<str>>,
    /// Native device_event_key string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_event_key: Option<Box<str>>,
    /// Native date_received string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_received: Option<Box<str>>,
    /// Native other_id_number string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_id_number: Option<Box<str>>,
    /// Native combination_product_flag string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combination_product_flag: Option<Box<str>>,
    /// Native serviced_by_3rd_party_flag string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serviced_by_3rd_party_flag: Option<Box<str>>,
}
/// Whole unchanged narrative row; oversized rows require a too-large Problem.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventNarrativeItem {
    /// Absolute native narrative index.
    pub index: u32,
    /// Native mdr_text_key string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mdr_text_key: Option<Box<str>>,
    /// Native patient_sequence_number string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patient_sequence_number: Option<Box<str>>,
    /// Native narrative text type code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Box<str>>,
    /// Unchanged complete source text; never silently truncated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<Box<str>>,
}
