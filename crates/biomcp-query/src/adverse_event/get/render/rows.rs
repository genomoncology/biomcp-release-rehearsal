use biodata::{FaersDrug, FaersReaction};
use biomcp_adapters::NativePositions;
use biomcp_types::{
    AdverseEventDrug, AdverseEventGet, AdverseEventIdentityCounts, AdverseEventReaction,
};
pub(super) fn drugs(
    q: &AdverseEventGet,
    rows: &[FaersDrug],
    positions: Option<&NativePositions>,
) -> Box<[AdverseEventDrug]> {
    NativePositions::enumerate(rows, positions)
        .filter(|(index, _)| {
            *index >= q.section_offset()
                && *index < q.section_offset().saturating_add(q.section_limit())
        })
        .map(|(index, r)| AdverseEventDrug {
            index,
            medicinalproduct: r.medicinalproduct.clone(),
            role: r.drugcharacterization.clone(),
            indication: r.drugindication.clone(),
            drugadministrationroute: r.drugadministrationroute.clone(),
            drugauthorizationnumb: r.drugauthorizationnumb.clone(),
            drugbatchnumb: r.drugbatchnumb.clone(),
            drugcumulativedosagenumb: r.drugcumulativedosagenumb.clone(),
            drugcumulativedosageunit: r.drugcumulativedosageunit.clone(),
            drugdosageform: r.drugdosageform.clone(),
            drugdosagetext: r.drugdosagetext.clone(),
            drugenddate: r.drugenddate.clone(),
            drugenddateformat: r.drugenddateformat.clone(),
            drugintervaldosagedefinition: r.drugintervaldosagedefinition.clone(),
            drugintervaldosageunitnumb: r.drugintervaldosageunitnumb.clone(),
            drugrecurreadministration: r.drugrecurreadministration.clone(),
            drugseparatedosagenumb: r.drugseparatedosagenumb.clone(),
            drugstartdate: r.drugstartdate.clone(),
            drugstartdateformat: r.drugstartdateformat.clone(),
            drugstructuredosagenumb: r.drugstructuredosagenumb.clone(),
            drugstructuredosageunit: r.drugstructuredosageunit.clone(),
            drugtreatmentduration: r.drugtreatmentduration.clone(),
            drugtreatmentdurationunit: r.drugtreatmentdurationunit.clone(),
            actiondrug: r.actiondrug.clone(),
            drugadditional: r.drugadditional.clone(),
            identity_counts: counts(r.openfda.as_ref()),
        })
        .collect()
}
fn counts(r: Option<&biodata::FaersIdentifiers>) -> AdverseEventIdentityCounts {
    AdverseEventIdentityCounts {
        unii: r.and_then(|v| v.unii.as_ref()).map(|v| v.len() as u32),
        rxcui: r.and_then(|v| v.rxcui.as_ref()).map(|v| v.len() as u32),
        product_ndc: r
            .and_then(|v| v.product_ndc.as_ref())
            .map(|v| v.len() as u32),
        package_ndc: r
            .and_then(|v| v.package_ndc.as_ref())
            .map(|v| v.len() as u32),
        spl_id: r.and_then(|v| v.spl_id.as_ref()).map(|v| v.len() as u32),
        spl_set_id: r
            .and_then(|v| v.spl_set_id.as_ref())
            .map(|v| v.len() as u32),
        application_number: r
            .and_then(|v| v.application_number.as_ref())
            .map(|v| v.len() as u32),
    }
}
pub(super) fn reactions(
    q: &AdverseEventGet,
    rows: &[FaersReaction],
    positions: Option<&NativePositions>,
) -> Box<[AdverseEventReaction]> {
    NativePositions::enumerate(rows, positions)
        .filter(|(index, _)| {
            *index >= q.section_offset()
                && *index < q.section_offset().saturating_add(q.section_limit())
        })
        .map(|(index, r)| AdverseEventReaction {
            index,
            term: r.term.clone(),
            vocabulary_version: r.vocabulary_version.clone(),
            outcome: r.outcome.clone(),
        })
        .collect()
}
