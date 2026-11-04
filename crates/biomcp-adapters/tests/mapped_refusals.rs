//! Source contracts exercise the adapters that own native admission and mapping.
#[path = "mapped_refusals/mod.rs"]
mod mapped_refusals;
use biomcp_test_support::network_stub;
use biomcp_test_support::replay as replay_support;
use biomcp_test_support::value;
use libtest_mimic::{Failed, Trial};
fn trial(name: &str, run: impl FnOnce() -> Result<(), Failed> + Send + 'static) -> Trial {
    Trial::test(name, run)
}
fn main() {
    libtest_mimic::run(
        &libtest_mimic::Arguments::from_args(),
        mapped_refusals::trials(),
    )
    .exit();
}
