//! Reviewed study-purpose and noncommercial flags; unknown studies remain enabled.
/// Reviewed reason for withholding a study from commercial-only study reads.
/// Refresh this checked-in list when reviewing a changed public study catalog.
pub fn commercial_restriction(id: &str) -> Option<&'static str> {
    include_str!("commercial-restrictions.tsv")
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            Some((fields.next()?, fields.next()?))
        })
        .find_map(|(study, reason)| (study == id).then_some(reason))
}
