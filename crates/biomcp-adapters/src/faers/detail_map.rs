use super::detail_raw as raw;
use biodata::{
    FaersDrug, FaersIdentifiers, FaersPatient, FaersReaction, FaersReport, FaersReporter,
    FaersSeriousness,
};
type Patient = (
    Option<FaersPatient>,
    Box<[FaersDrug]>,
    Box<[FaersReaction]>,
    super::Positions,
);
pub(super) fn report(r: raw::Report) -> crate::Mapped<super::positions::Report> {
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Faers,
        record: crate::mapped::record(&r.safetyreportid),
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let seriousness = FaersSeriousness {
        death: problems.field("seriousnessdeath", r.seriousnessdeath),
        lifethreatening: problems.field("seriousnesslifethreatening", r.seriousnesslifethreatening),
        hospitalization: problems.field("seriousnesshospitalization", r.seriousnesshospitalization),
        disabling: problems.field("seriousnessdisabling", r.seriousnessdisabling),
        congenitalanomali: problems.field(
            "seriousnesscongenitalanomali",
            r.seriousnesscongenitalanomali,
        ),
        other: problems.field("seriousnessother", r.seriousnessother),
    };
    let has_flags = seriousness.death.is_some()
        || seriousness.lifethreatening.is_some()
        || seriousness.hospitalization.is_some()
        || seriousness.disabling.is_some()
        || seriousness.congenitalanomali.is_some()
        || seriousness.other.is_some();
    let raw_patient = problems.field("patient", r.patient);
    let (patient, drugs, reactions, positions) = match raw_patient {
        Some(p) => patient(p, &mut problems, &mut refusals),
        None => (
            None,
            Box::default(),
            Box::default(),
            super::Positions::default(),
        ),
    };
    let value = FaersReport {
        safetyreportid: r.safetyreportid,
        safetyreportversion: problems.field("safetyreportversion", r.safetyreportversion),
        receivedate: problems.field("receivedate", r.receivedate),
        receivedateformat: problems.field("receivedateformat", r.receivedateformat),
        receiptdate: problems.field("receiptdate", r.receiptdate),
        receiptdateformat: problems.field("receiptdateformat", r.receiptdateformat),
        transmissiondate: problems.field("transmissiondate", r.transmissiondate),
        transmissiondateformat: problems.field("transmissiondateformat", r.transmissiondateformat),
        serious: problems.field("serious", r.serious),
        seriousness: has_flags.then_some(seriousness),
        patient,
        reporter: problems
            .field("primarysource", r.primarysource)
            .map(|s| FaersReporter {
                qualification: problems.field("qualification", s.qualification),
                country: problems.field("reportercountry", s.reportercountry),
            }),
        drugs,
        reactions,
    };
    crate::Mapped {
        refusals,
        value: super::positions::Report {
            data: value,
            positions,
        },
        problems: problems.list,
    }
}
fn patient(
    p: raw::Patient,
    problems: &mut crate::mapped::Problems,
    refusals: &mut crate::Refusal,
) -> Patient {
    let patient = Some(FaersPatient {
        age: age(problems, p.patientonsetage, p.patientonsetageunit),
        sex: problems.field("patient.patientsex", p.patientsex),
        weight: problems.field("patient.patientweight", p.patientweight),
    });
    let (drugs, drug_positions) = problems.indexed_fields("patient.drug", p.drug, refusals);
    let drugs = drugs
        .into_vec()
        .into_iter()
        .map(|v| drug(v, problems))
        .collect();
    let (reactions, reaction_positions) =
        problems.indexed_fields("patient.reaction", p.reaction, refusals);
    let reactions = reactions
        .into_vec()
        .into_iter()
        .map(|v| FaersReaction {
            term: problems
                .source_field("patient.reaction.reactionmeddrapt", v.reactionmeddrapt)
                .map(|label| biodata::Concept { id: None, label }),
            vocabulary_version: problems.field(
                "patient.reaction.reactionmeddraversionpt",
                v.reactionmeddraversionpt,
            ),
            outcome: problems.field("patient.reaction.reactionoutcome", v.reactionoutcome),
        })
        .collect();
    (
        patient,
        drugs,
        reactions,
        super::Positions {
            drugs: drug_positions,
            reactions: reaction_positions,
        },
    )
}
fn identifiers(r: raw::Identifiers, p: &mut crate::mapped::Problems) -> FaersIdentifiers {
    FaersIdentifiers {
        unii: p.optional_fields("unii", r.unii).map(Vec::into_boxed_slice),
        rxcui: p
            .optional_fields("rxcui", r.rxcui)
            .map(Vec::into_boxed_slice),
        product_ndc: p
            .optional_fields("product_ndc", r.product_ndc)
            .map(Vec::into_boxed_slice),
        package_ndc: p
            .optional_fields("package_ndc", r.package_ndc)
            .map(Vec::into_boxed_slice),
        spl_id: p
            .optional_fields("spl_id", r.spl_id)
            .map(Vec::into_boxed_slice),
        spl_set_id: p
            .optional_fields("spl_set_id", r.spl_set_id)
            .map(Vec::into_boxed_slice),
        application_number: p
            .optional_fields("application_number", r.application_number)
            .map(Vec::into_boxed_slice),
    }
}
fn drug(r: raw::Drug, p: &mut crate::mapped::Problems) -> FaersDrug {
    FaersDrug {
        medicinalproduct: p.field("medicinalproduct", r.medicinalproduct),
        drugcharacterization: p.field("drugcharacterization", r.drugcharacterization),
        drugindication: p.field("drugindication", r.drugindication),
        drugadministrationroute: p.field("drugadministrationroute", r.drugadministrationroute),
        drugauthorizationnumb: p.field("drugauthorizationnumb", r.drugauthorizationnumb),
        drugbatchnumb: p.field("drugbatchnumb", r.drugbatchnumb),
        drugcumulativedosagenumb: p.field("drugcumulativedosagenumb", r.drugcumulativedosagenumb),
        drugcumulativedosageunit: p.field("drugcumulativedosageunit", r.drugcumulativedosageunit),
        drugdosageform: p.field("drugdosageform", r.drugdosageform),
        drugdosagetext: p.field("drugdosagetext", r.drugdosagetext),
        drugenddate: p.field("drugenddate", r.drugenddate),
        drugenddateformat: p.field("drugenddateformat", r.drugenddateformat),
        drugintervaldosagedefinition: p.field(
            "drugintervaldosagedefinition",
            r.drugintervaldosagedefinition,
        ),
        drugintervaldosageunitnumb: p
            .field("drugintervaldosageunitnumb", r.drugintervaldosageunitnumb),
        drugrecurreadministration: p
            .field("drugrecurreadministration", r.drugrecurreadministration),
        drugseparatedosagenumb: p.field("drugseparatedosagenumb", r.drugseparatedosagenumb),
        drugstartdate: p.field("drugstartdate", r.drugstartdate),
        drugstartdateformat: p.field("drugstartdateformat", r.drugstartdateformat),
        drugstructuredosagenumb: p.field("drugstructuredosagenumb", r.drugstructuredosagenumb),
        drugstructuredosageunit: p.field("drugstructuredosageunit", r.drugstructuredosageunit),
        drugtreatmentduration: p.field("drugtreatmentduration", r.drugtreatmentduration),
        drugtreatmentdurationunit: p
            .field("drugtreatmentdurationunit", r.drugtreatmentdurationunit),
        actiondrug: p.field("actiondrug", r.actiondrug),
        drugadditional: p.field("drugadditional", r.drugadditional),
        openfda: p
            .field("patient.drug.openfda", r.openfda)
            .map(|v| identifiers(v, p)),
    }
}

fn age(
    p: &mut crate::mapped::Problems,
    value: Option<crate::raw::Field<String>>,
    unit: Option<crate::raw::Field<biodata::FaersAgeUnit>>,
) -> Option<biodata::WithUnit<biodata::FiniteNumber, biodata::FaersAgeUnit>> {
    let supplied = unit.is_some();
    let unit = p.field("patient.patientonsetageunit", unit);
    let raw = p.field("patient.patientonsetage", value)?;
    let value = p.optional(
        "patient.patientonsetage",
        Some(
            raw.parse::<f64>()
                .map_err(|_| ())
                .and_then(|n| biodata::FiniteNumber::try_from(n).map_err(|_| ())),
        ),
    )?;
    if supplied && unit.is_none() {
        return None;
    }
    Some(biodata::WithUnit { value, unit })
}
