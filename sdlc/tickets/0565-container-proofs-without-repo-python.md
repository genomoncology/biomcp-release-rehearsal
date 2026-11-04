# Container check runs tool proofs without repository Python

Status: COMPLETE.
Landed: 3bc6918b4a40861537233aeb347a15aabce6dff5
Depends on: 0562
Milestone: 5

## Outcome

The shipped container image stays free of Python. The container check runs every tool proof that tests the shipped binary. The one proof that checks the repository's nightly Python classifier, `live-source-checks/retained-drift-witnesses`, keeps running from the checkout and from the installed wheel gate, where Python is present. The container check leaves it out by a declared, tested exclusion, not by a silent skip.

- Why: run j25622 showed the pinned runtime image has no `python3` (exit 127; Yellow `biomcp-redux-evidence/0562/j25622-runtime-python.stdout`). `release/Dockerfile` installs only certificates, OpenSSL and biomcp, and `release/container_check.py` runs every tool proof, including the one that calls `scripts/icite_discovery_proof.py`. That helper imports `classify_case` from `scripts/nightly_live_check.py`. It proves the repository's own nightly tool against the binary's observations, which is a developer-tool contract, not a runtime one. Porting it to Rust would stop testing the unchanged classifier, and adding Python to the image would ship a dependency for a test.
- Starts from: `release/container_check.py`, `crates/biomcp/tests/runner/icite_discovery.rs`, `crates/biomcp/tests/runner/live_proof.rs:23`, the tool runner's proof selection, `release/test_linux.py`.
- Rules: a proof declares that it needs repository Python tooling. The container check passes an exclusion of exactly the declared proofs and prints their names and count in its report. A test fails if the exclusion list grows past the declared set, or if the container check runs zero proofs. Checkout and installed wheel runs still run every proof. No Python goes into the production image, and no proof is weakened.
- Proof: a fixture test of the container check's proof selection and report; the next private rehearsal's container check passes on Linux ARM and x64 with one named exclusion.
- Changes: proof metadata, the tool runner's selection, `release/container_check.py` and its tests.
- Defers: the helper staging for installed runs (iCite helper ticket) and the evidence paths (Linux evidence ticket).
