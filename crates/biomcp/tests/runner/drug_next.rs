//! Feature drug-next-steps: wholly synthetic D05/D17/D19-shaped source contexts.
//! These fixtures test selector propagation, never clinical answers or licensed annotations.
use libtest_mimic::Trial;
mod native;

pub(super) fn trials() -> Vec<Trial> {
    native::trials()
}
