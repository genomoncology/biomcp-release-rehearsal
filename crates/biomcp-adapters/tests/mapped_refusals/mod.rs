//! Actual record refusals stay distinct from optional fields at the public adapter boundary.
mod documents;
mod families;
mod members;
mod pages;
mod records;
mod retained;
mod source_pages;
mod stored;
mod synthetic;
use crate::trial;
use biomcp_adapters::{NativeWindow, Refusal, chembl};
use libtest_mimic::Trial;
use serde_json::json;

pub(super) fn trials() -> Vec<Trial> {
    let mut tests = vec![
        trial(
            "drug-native-sources/native-identity-refusals-carry-facts",
            || chembl_pages().map_err(Into::into),
        ),
        trial(
            "drug-native-sources/refusal-facts-follow-selected-native-slots",
            || selected_slots().map_err(Into::into),
        ),
    ];
    tests.extend(families::trials());
    tests.extend(pages::trials());
    tests.extend(members::trials());
    tests.extend(records::trials());
    tests.extend(synthetic::trials());
    tests.extend(source_pages::trials());
    tests.extend(stored::trials());
    tests.extend(documents::trials());
    tests.extend(retained::trials());
    tests
}

fn chembl_pages() -> Result<(), String> {
    for invalid in [false, true] {
        let molecule: chembl::Molecules = serde_json::from_value(json!({
            "molecules":[{
                "molecule_chembl_id": if invalid { "invalid" } else { "CHEMBL1" },
                "pref_name":{"bad_shape":true}, "molecule_type":"Small molecule"
            }],
            "page_meta":{"offset":0,"limit":1,"total_count":2}
        }))
        .map_err(|e| e.to_string())?;
        let mechanism: chembl::Mechanisms = serde_json::from_value(json!({
            "mechanisms":[{
                "molecule_chembl_id":"CHEMBL1", "mec_id": if invalid { 0 } else { 1 },
                "mechanism_of_action":{"bad_shape":true}
            }],
            "page_meta":{"offset":0,"limit":1,"total_count":2}
        }))
        .map_err(|e| e.to_string())?;
        let indication: chembl::Indications = serde_json::from_value(json!({
            "drug_indications":[{
                "molecule_chembl_id":"CHEMBL1", "drugind_id": if invalid { 0 } else { 1 },
                "max_phase_for_ind":{"bad_shape":true}
            }],
            "page_meta":{"offset":0,"limit":1,"total_count":2}
        }))
        .map_err(|e| e.to_string())?;
        for (name, refusals, rows, next) in [
            (
                "molecule",
                molecule.window.refusals(),
                molecule.window.rows.len(),
                molecule.window.next_offset(),
            ),
            (
                "mechanism",
                mechanism.window.refusals(),
                mechanism.window.rows.len(),
                mechanism.window.next_offset(),
            ),
            (
                "indication",
                indication.window.refusals(),
                indication.window.rows.len(),
                indication.window.next_offset(),
            ),
        ] {
            let expected = if invalid {
                Refusal::Refused
            } else {
                Refusal::None
            };
            if refusals != expected || rows != usize::from(!invalid) || next != Some(1) {
                return Err(format!(
                    "{name} lost its actual refusal decision or native continuation"
                ));
            }
        }
        if !invalid
            && (molecule
                .window
                .rows
                .first()
                .is_none_or(|row| row.problems().is_empty())
                || mechanism
                    .window
                    .rows
                    .first()
                    .is_none_or(|row| row.problems().is_empty())
                || indication
                    .window
                    .rows
                    .first()
                    .is_none_or(|row| row.problems().is_empty()))
        {
            return Err("optional-field control did not produce retained-row problems".into());
        }
    }
    Ok(())
}

fn selected_slots() -> Result<(), String> {
    let refused = NativeWindow::local(vec![Some(1), None, Some(3)], 1, 1);
    let retained = NativeWindow::local(vec![Some(1), None, Some(3)], 2, 1);
    if refused.refusals() != Refusal::Refused
        || !refused.rows.is_empty()
        || refused.received() != 1
        || refused.next_offset() != Some(2)
        || retained.refusals() != Refusal::None
        || retained.rows != [3]
        || retained.next_offset().is_some()
    {
        return Err("refusals escaped selected native slots or changed continuation".into());
    }
    let projected = refused.with_rows(Vec::<String>::new());
    if projected.refusals() != Refusal::Refused || projected.next_offset() != Some(2) {
        return Err("row projection lost the source window's refusal".into());
    }
    Ok(())
}
