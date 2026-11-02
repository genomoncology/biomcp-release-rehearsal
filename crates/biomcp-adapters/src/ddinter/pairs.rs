use super::NativeRow;
use std::collections::BTreeMap;
/// An original native row's location in one published partition.
#[derive(Debug)]
struct Provenance {
    /// Native partition code, independent of local file paths.
    part: Box<str>,
    /// One-based logical CSV row, including the header.
    row: u64,
}
/// A checked row and its original location in the complete replacement set.
pub(super) type PartRow = biomcp_fetch::SetRow<NativeRow>;
/// One native pair oriented to the selected source ID.
#[derive(Debug)]
pub(super) struct Pair {
    /// The first complete native pair row, without merged wording.
    native: NativeRow,
    /// Every original row location contributing this same source pair.
    provenance: Vec<Provenance>,
    reversed: bool,
}
impl Pair {
    /// Map the complete native pair with primary and distinct further source origins.
    pub(super) fn claim(
        &self,
        content_revision: Option<&str>,
    ) -> Result<crate::Mapped<biodata::DrugInteractionClaim>, biodata::Error> {
        let anchor_id = if self.reversed {
            self.native.id_b()
        } else {
            self.native.id_a()
        };
        let mut origins = self
            .provenance
            .iter()
            .map(|origin| attribution(origin, content_revision));
        let primary = origins.next().ok_or(biodata::Error::TextEmpty)??;
        let additional = origins.collect::<Result<Vec<_>, _>>()?;
        let problems = self.native.problems(primary.record.clone());
        let value = biodata::DrugInteractionClaim::new(
            super::identity::identity(anchor_id, self.anchor_name())?,
            super::identity::identity(self.partner_id(), self.partner_name())?,
            self.native.level().cloned(),
            primary,
            additional.into_boxed_slice(),
        )?;
        Ok(crate::Mapped {
            refusals: crate::Refusal::None,
            value,
            problems,
        })
    }
    /// Exact native partner ID, without a name join.
    fn partner_id(&self) -> &str {
        if self.reversed {
            self.native.id_a()
        } else {
            self.native.id_b()
        }
    }
    /// Native partner display name from the retained row.
    fn partner_name(&self) -> Option<&str> {
        if self.reversed {
            self.native.name_a()
        } else {
            self.native.name_b()
        }
    }
    /// Native anchor name from the retained row.
    fn anchor_name(&self) -> Option<&str> {
        if self.reversed {
            self.native.name_b()
        } else {
            self.native.name_a()
        }
    }
}
/// Select exact source partners in partition/row order, retaining every duplicate origin.
pub(super) fn partners(mut rows: Vec<PartRow>, anchor: &str) -> Vec<Pair> {
    rows.sort_by(|a, b| (&a.part, a.row).cmp(&(&b.part, b.row)));
    let mut pairs = BTreeMap::<String, Pair>::new();
    let mut order = Vec::new();
    for row in rows {
        let reversed = row.value.id_b() == anchor;
        if row.value.id_a() != anchor && !reversed {
            continue;
        }
        let partner = if reversed {
            row.value.id_a()
        } else {
            row.value.id_b()
        };
        let origin = Provenance {
            part: row.part,
            row: row.row,
        };
        if let Some(pair) = pairs.get_mut(partner) {
            pair.provenance.push(origin);
        } else {
            order.push(partner.to_owned());
            pairs.insert(
                partner.to_owned(),
                Pair {
                    native: row.value,
                    provenance: vec![origin],
                    reversed,
                },
            );
        }
    }
    order
        .into_iter()
        .filter_map(|id| pairs.remove(&id))
        .collect()
}

fn attribution(
    origin: &Provenance,
    revision: Option<&str>,
) -> Result<biodata::Attribution, biodata::Error> {
    use biodata::Text;
    let path = super::bundle::PARTS
        .iter()
        .find(|part| part.name == origin.part.as_ref())
        .map(|part| part.path)
        .ok_or(biodata::Error::TextEmpty)?;
    let filename = path.rsplit('/').next().ok_or(biodata::Error::TextEmpty)?;
    Ok(biodata::Attribution {
        approved: None,
        published: None,
        source: Text::new("ddinter")?,
        record: Some(Text::new(&format!("{filename}:{}", origin.row))?),
        version: revision.map(Text::new).transpose()?,
        release: None,
        submitter: None,
        evaluated: None,
        url: Some(Text::new(&format!("https://ddinter.scbdd.com{path}"))?),
    })
}
