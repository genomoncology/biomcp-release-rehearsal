# Windows tool proofs find Python

Status: OPEN.
Depends on: 0562
Milestone: 5

## Outcome

On Windows, the installed wheel gate runs `live-source-checks/retained-drift-witnesses` with an explicit, working Python interpreter, so the proof runs there rather than failing with "program not found".

- Why: private rehearsal run 41358845085 at ab41e914f failed this proof on Windows with program not found (Yellow `biomcp-redux-evidence/0294/private-rehearsal-41358845085/windows-job.log` lines 982-983). `crates/biomcp/tests/runner/icite_discovery.rs` runs `python3`, which a standard Windows Python install does not provide. The container exclusion in the container proof ticket does not cover Windows: the Windows installed gate has Python and must still run this proof.
- Starts from: `crates/biomcp/tests/runner/icite_discovery.rs`, `release/gate.py` (which already knows its interpreter through `--python`), `.github/workflows/release.yml` Windows qualification.
- Rules: the gate passes the interpreter it was given to the tool runner, for example through an environment variable that the proof reads, and the proof fails clearly when none is given rather than guessing. Do not skip the proof on Windows.
- Proof: a fixture test that the gate hands its interpreter to the proof, and the next private rehearsal confirms Windows, Linux and macOS.
- Changes: the proof's interpreter lookup and the gate's handoff only.
- Defers: helper staging (iCite helper ticket) and the container exclusion (container proof ticket).
