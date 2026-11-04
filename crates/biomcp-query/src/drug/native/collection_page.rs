//! Project accepted collection rows without transferring collection-owned refusal facts.
use biomcp_adapters::NativeWindow;
// Accepted-row offsets belong to this legacy public projection. Refusal events belong
// to the complete Mapped collection, which this borrow preserves for the section owner.
pub(super) fn page<T>(
    collection: &mut biomcp_adapters::Mapped<Box<[T]>>,
    offset: usize,
    limit: usize,
) -> NativeWindow<T> {
    let total = collection.value.len() as u64;
    let rows = std::mem::take(&mut collection.value)
        .into_vec()
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    NativeWindow::new(
        rows,
        offset as u32,
        limit as u32,
        Some(total),
        total.saturating_sub(offset as u64).min(limit as u64) as u32,
        biomcp_adapters::Refusal::None,
    )
}
