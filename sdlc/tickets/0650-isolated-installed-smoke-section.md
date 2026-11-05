# Run the full installed smoke as one observable, isolated section

Status: OPEN.

Milestone: 5

## Outcome

The existing installed cleanup smoke runs independently against retained, source-bound release artifacts and matching runners, with its original time limits and enough owned phase evidence to identify a failure. Land the already reviewed and exercised tooling from `ticket/0066-full-installed-smoke`, without changing product code (Ian's independent-section ruling, m26087).

## Why

The earlier broad debug run j30338 timed out after 300 seconds with buffered child output; subsequent isolated attempts exposed missing fixture environment and helper staging. The corrected section passed j30464 against product `1bb97e64f` in 258.393 seconds.

## Proof

- Proof: carry the reviewed T `4f2e13b07cc2dd8d92c0731de2480bbc3718daf1` tooling/preflight and actual j30464 evidence when bytes remain unchanged: 611 cases, tool proofs, eight-process startup, HTTP/TLS, cleanup and sentinel assertions. Keep all earlier red results separate; no repeat of passing qualification just to move the source to its ticket branch.

## Starts from

- Starts from: the clean pushed `ticket/0066-full-installed-smoke` branch, accepted source T above, and Yellow `data/biomcp-redux-evidence/0293/candidate-1bb97e64f416/full-installed-smoke-T4f2e13b07/runtime-j30464`. Witness SHA256: `9de3985b9a7c344baba020f3997fd2a949ed5fd6b022d608d7df03428d135e46`.

## Keeps

- Keeps: exact product/tooling identities and runtime fixture byte bindings; canonical fixture staging; existing 300/60-second limits; no live database observer; only owned process-group cleanup; no environment, argument or credential dumps; no Rust, Cargo, schema or product changes.

## Defers

- Defers: product changes and broader release orchestration. This is tooling-only and carries product Q proof under m26263.

## Changes

- Changes: `release/smoke.py`, `release/test_smoke.py`, `scripts/installed_smoke_section.py`, `scripts/test_installed_smoke_section.py`; the existing reviewed phase instrumentation, fixture staging and standalone launcher.
