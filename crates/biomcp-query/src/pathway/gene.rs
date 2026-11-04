use crate::{Ctx, Outcome, gene, status};
use biodata::{PathwayId, PathwaySpecies};
use biomcp_adapters::{Mapped, hgnc, reactome};
use biomcp_fetch::{Deadline, InputError, RequestError};
use biomcp_types::{GeneResolution, PathwayHeader, PathwaySearch, Problem, ProblemCode, Source};
pub(super) fn headers(
    cx: &Ctx<'_>,
    q: &PathwaySearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Mapped<Vec<PathwayHeader>>, RequestError> {
    let input = q.gene().ok_or(InputError::Serialization)?;
    let identity = cx
        .fetch
        .leg(&hgnc::SPEC, deadline)
        .fresh(cx.fresh || q.fresh());
    let resolved = gene::resolve(&identity, input);
    out.sources.push(status::source(
        Source::Hgnc,
        identity.finish(),
        resolved.as_ref().err().copied(),
    ));
    let resolved = resolved?;
    out.problems.extend(resolved.problems);
    let GeneResolution::Resolved { gene, .. } = resolved.value else {
        return Ok(Mapped {
            refusals: resolved.refusals,
            value: Vec::new(),
            problems: vec![Problem {
                source: Source::Hgnc,
                record: input.to_string().parse().ok(),
                field: "gene".into(),
                code: ProblemCode::ValueUnknown,
            }],
        });
    };
    let leg = cx
        .fetch
        .leg(&reactome::SPEC, deadline)
        .fresh(cx.fresh || q.fresh());
    let replies = leg.batch(&gene.uniprot, reactome::pathways);
    let error = replies.iter().find_map(|r| r.as_ref().err().copied());
    out.sources
        .push(status::source(Source::Reactome, leg.finish(), error));
    let mut mapped = Mapped {
        refusals: resolved.refusals,
        value: Vec::new(),
        problems: Vec::new(),
    };
    for reply in replies {
        let rows = reply?;
        mapped.refusals.include(rows.refusals);
        mapped.problems.extend(rows.problems);
        for row in rows.value {
            let id = PathwayId::Reactome(row.id);
            if !mapped.value.iter().any(|r: &PathwayHeader| r.id == id) {
                mapped.value.push(PathwayHeader {
                    id,
                    name: row.name,
                    species: PathwaySpecies::Human,
                    version: None,
                    revision: None,
                });
            }
        }
    }
    if gene.uniprot.is_empty() {
        mapped.problems.push(Problem {
            source: Source::Hgnc,
            record: Some(
                gene.hgnc_id
                    .to_string()
                    .parse()
                    .map_err(|_| InputError::Serialization)?,
            ),
            field: "uniprot".into(),
            code: ProblemCode::ValueUnknown,
        });
    }
    Ok(mapped)
}
