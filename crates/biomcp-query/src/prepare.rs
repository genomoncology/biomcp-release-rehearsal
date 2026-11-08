//! Capability and selector checks run before transport admission.
use crate::Action;
use biomcp_types::{DiseaseSection, DiseaseSource, Request, UsageError};
impl Action {
    /// Prepare source selectors and preserve existing usage refusal envelopes.
    pub fn prepare(self) -> Result<Self, UsageError> {
        let action = prepare_disease(self).map_err(selector_error)?;
        drug_selector(&action)?;
        if let Some(error) = unsupported(&action) {
            return Err(error);
        }
        Ok(action)
    }
}
fn disease_unsupported(action: &Action) -> bool {
    match action {
        Action::Run {
            request: Request::SearchDisease(q),
            ..
        } => !matches!(q.source(), DiseaseSource::Ols | DiseaseSource::Seer),
        Action::Run {
            request: Request::GetDisease(q),
            ..
        } => match q.source() {
            DiseaseSource::Ols => !matches!(
                q.section(),
                None | Some(DiseaseSection::Synonyms | DiseaseSection::Xrefs)
            ),
            DiseaseSource::MyDisease => {
                let supported = biomcp_adapters::mydisease::supported_id(q.id().as_str());
                !supported
                    || !matches!(
                        q.section(),
                        None | Some(
                            DiseaseSection::Synonyms
                                | DiseaseSection::Xrefs
                                | DiseaseSection::Hierarchy
                        )
                    )
            }
            DiseaseSource::Monarch => !matches!(
                q.section(),
                Some(DiseaseSection::Genes | DiseaseSection::Phenotypes)
            ),
            DiseaseSource::Seer => !matches!(
                q.section(),
                Some(DiseaseSection::Incidence | DiseaseSection::Survival)
            ),
        },
        _ => false,
    }
}
fn prepare_disease(action: Action) -> Result<Action, biomcp_adapters::seer::SelectorError> {
    if let Action::Run {
        request: Request::GetDisease(query),
        fresh,
    } = action
    {
        let biomcp_types::DiseaseId::SeerSite(id) = query.id() else {
            return Ok(Action::Run {
                request: Request::GetDisease(query),
                fresh,
            });
        };
        let selection = biomcp_adapters::seer::selection(&query, id)?;
        let query = query
            .with_demographics(selection.sex, selection.race, selection.age_range)
            .map_err(|_| biomcp_adapters::seer::SelectorError {
                selectors: vec![biomcp_adapters::seer::Selector::Sex],
            })?;
        return Ok(Action::Run {
            request: Request::GetDisease(query),
            fresh,
        });
    }
    Ok(action)
}
pub(crate) fn selector_error(error: biomcp_adapters::seer::SelectorError) -> UsageError {
    let values = error
        .selectors
        .iter()
        .flat_map(|selector| {
            selector.valid_codes().iter().map(|code| {
                if error.selectors.len() == 1 {
                    (*code).to_string()
                } else {
                    format!("{} {}", selector.flag(), code)
                }
            })
        })
        .map(|code| code.parse())
        .collect::<Result<Vec<biodata::Text>, _>>();
    let Ok(values) = values else {
        return UsageError::arguments();
    };
    UsageError::SourceSelector {
        source: biomcp_types::Source::Seer,
        message: biomcp_types::ErrorMessage::SourceSelector,
        option: error
            .selectors
            .iter()
            .map(|s| s.flag())
            .collect::<Vec<_>>()
            .join(", ")
            .into(),
        valid_values: values.into(),
        next: vec![biomcp_types::Step {
            kind: biomcp_types::StepKind::Disambiguate,
            args: ["search", "disease", "--source", "seer"]
                .map(Into::into)
                .into(),
        }],
    }
}
fn unsupported(action: &Action) -> Option<UsageError> {
    if let Action::Run {
        request: Request::GetCellLine(q),
        ..
    } = action
        && matches!(q.id(), biomcp_types::CellLineInput::Source(id) if biomcp_adapters::cellosaurus::native_database(id).is_none())
    {
        return Some(UsageError::arguments());
    }
    if let Action::Run {
        request: Request::GetCellLine(query),
        ..
    } = action
        && query.section() == Some(biomcp_types::CellLineSection::All)
        && (query.section_limit().is_some() || query.section_offset() != 0)
    {
        return Some(UsageError::message(
            biomcp_types::ErrorMessage::CellLineAllPaging,
        ));
    }
    if disease_unsupported(action) {
        return Some(UsageError::arguments());
    }
    if let Action::Run {
        request: Request::SearchTrial(q),
        ..
    } = action
    {
        if q.nci_disease_invalid() {
            return Some(UsageError::NciDiseaseError);
        }
        if let Some(option) = q.unsupported() {
            let source = if q.source() == biomcp_types::TrialSource::Nci {
                biomcp_types::Source::Nci
            } else {
                biomcp_types::Source::ClinicalTrialsGov
            };
            return Some(UsageError::UnsupportedTrialFilter { source, option });
        }
    }

    if let Action::Run {
        request: Request::SearchArticle(query),
        ..
    } = action
        && let Some((source, option)) = crate::article::unsupported(query)
    {
        return Some(UsageError::UnsupportedFilter { source, option });
    }
    None
}

fn drug_selector(action: &Action) -> Result<(), UsageError> {
    let Action::Run {
        request: Request::GetDrug(query),
        ..
    } = action
    else {
        return Ok(());
    };
    let label_only = query.sections() == [biomcp_types::DrugSection::Label];
    let (option, values) = if !query.supports_sections() && label_only {
        // Only the openFDA label source serves a label read, so name the source to pass.
        ("source", "openfda-label".parse().map(|v| vec![v]))
    } else if !query.supports_sections() {
        (
            "section",
            query
                .source()
                .sections()
                .iter()
                .map(|section| <&str>::from(*section).parse())
                .collect::<Result<Vec<biodata::Text>, _>>(),
        )
    } else {
        return Ok(());
    };
    let values = values.map_err(|_| UsageError::arguments())?;
    Err(UsageError::SourceSelector {
        source: query.source().source(),
        message: if query.sections().len() > 1 {
            biomcp_types::ErrorMessage::OneSection
        } else {
            biomcp_types::ErrorMessage::SourceSelector
        },
        option: option.into(),
        valid_values: values.into(),
        next: vec![biomcp_types::Step {
            kind: biomcp_types::StepKind::Configure,
            args: ["help", "get", "drug"].map(Into::into).into(),
        }],
    })
}
