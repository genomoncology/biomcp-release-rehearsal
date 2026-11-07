//! Words for coded FAERS values, copied from openFDA's published field reference:
//! <https://open.fda.gov/apis/drug/event/searchable-fields/>. Native codes stay in every reply.

/// One published code list.
pub type Words = &'static [(&'static str, &'static str)];

/// `patient.reaction.reactionoutcome`.
pub const REACTION_OUTCOME: Words = &[
    ("1", "Recovered/resolved"),
    ("2", "Recovering/resolving"),
    ("3", "Not recovered/not resolved"),
    (
        "4",
        "Recovered/resolved with sequelae (consequent health issues)",
    ),
    ("5", "Fatal"),
    ("6", "Unknown"),
];
/// `patient.patientsex`.
pub const PATIENT_SEX: Words = &[("0", "Unknown"), ("1", "Male"), ("2", "Female")];
/// `patient.drug.drugcharacterization`.
pub const DRUG_ROLE: Words = &[
    (
        "1",
        "Suspect (the drug was considered by the reporter to be the cause)",
    ),
    (
        "2",
        "Concomitant (the drug was reported as being taken along with the suspect drug)",
    ),
    (
        "3",
        "Interacting (the drug was considered by the reporter to have interacted with the suspect drug)",
    ),
];
/// `patient.drug.actiondrug`.
pub const ACTION_TAKEN: Words = &[
    ("1", "Drug withdrawn"),
    ("2", "Dose reduced"),
    ("3", "Dose increased"),
    ("4", "Dose not changed"),
    ("5", "Unknown"),
    ("6", "Not applicable"),
];
/// `primarysource.qualification`.
pub const REPORTER_QUALIFICATION: Words = &[
    ("1", "Physician"),
    ("2", "Pharmacist"),
    ("3", "Other health professional"),
    ("4", "Lawyer"),
    ("5", "Consumer or non-health professional"),
];

/// The published word for a code; a code outside the list has none.
pub fn label(words: Words, code: Option<&str>) -> Option<Box<str>> {
    let code = code?;
    words
        .iter()
        .find(|(known, _)| *known == code)
        .map(|(_, word)| (*word).into())
}
