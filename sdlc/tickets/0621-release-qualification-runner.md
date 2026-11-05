# Release qualification exercises every channel it claims

Status: COMPLETE.
Landed: 07a9cb7f6a07142f30c647a897e1fc9341e392fe
Milestone: 5

## Outcome

The final release qualification really runs the channels it reports: the Windows wrappers check the current tickets, the Claude plugin is installed and runs one command, and the pinned 0.9 installer installs from the new release archives served locally. The five-platform native proof, Intel Mac included, comes from the already approved public hosted rehearsal of the final head.

- Why: m24937. At the final release head ed9c36a96, the build delegate found that the Windows wrappers validate older ticket IDs, the plugin tests check metadata without installing, and `release/published.py` fetches today's public script and archives, so it cannot test the pinned 0.9 installer against the new archives. No local machine has an Intel Mac toolchain (M5 runs arm64 Rust under Rosetta). A channel the qualification never ran must not be reported as passed.
- Starts from: `release/published.py`, `release/local_server.py`, `release/test_installer.py`, the Windows wrapper checks and the plugin tests under `release/`, and `release/rehearse.py` for the hosted rehearsal.
- Rules: proof tooling only; the frozen product source stays at 4250d1952. The legacy installer is served the new archives from `release/local_server.py`, never from the public site. The plugin install runs in a scratch home and is removed afterwards. The hosted rehearsal runs in the public rehearsal repository under a non-`v` tag with publishing off, as the 2026-11-02 decision allows. Each channel's result names its own evidence, and a channel that did not run is reported as not run.
- Proof: one qualification run at the final head shows the Windows wrapper, plugin install and legacy installer passing on their own evidence, and the hosted rehearsal shows all five native targets, Intel Mac included.
- Changes: `release/published.py`, `release/local_server.py`, `release/test_installer.py`, `release/test_promotion.py`, `release/rehearse.py`; the qualification steps and their tests.
- Defers: nothing.
