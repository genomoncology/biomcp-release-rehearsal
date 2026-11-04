# 0493 — Name the missing or bad option in every usage error

Status: COMPLETE.
Landed: 137247baba494b03088b21a6d27e2656a5fad0bd

Milestone: 5

Depends on: 0421

## Outcome

A usage error says which option is missing or wrong, what it accepts, and which help call shows the command's shape. Today `get drug imatinib label` returns only "expected a supported command with checked arguments", so agents read help to guess the missing option.

- Why: BioData's experiment 502 rerun on internal build e7913048b (m15537), finding B. 42 of 44 round 2 BioMCP-only cells read help, partly because usage errors named nothing; `--country` was refused the same way.
- Starts from: the typed `UsageError` and checked constructors from 0421, the grammar and the `biomcp.error` schema.
- Rules: the message comes from the typed error, never from rescanning argv; the error names one option and its accepted values or form; no guessing of what the user meant.
- Proof: cases for a missing required option, an unknown option, a bad enum value and a bad typed ID each fail on the base build and show the option, its accepted form and the help call.
- Changes: usage errors, the error schema if a field is added, cases and baseline.
- Defers: nothing.
