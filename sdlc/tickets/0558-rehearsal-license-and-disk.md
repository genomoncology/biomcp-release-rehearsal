# 0558 — Fix the license notices and schema proof disk use on hosted runners

Status: COMPLETE.
Landed: 0b01ea3f1cd3f3da7764abebe28728771a6daf60

Milestone: 5

## Outcome

The release's third-party license notices build on every hosted runner, and the docs check's schema proof fits in a hosted runner's disk.

- Why: the private no-op release rehearsal (run 41348267985, m19319). The Linux ARM job failed in cargo-about because borrow-or-share 0.2.4 (MIT-0, through fluent-uri) and foldhash 0.2.0 (Zlib, through hashbrown) carry licenses that the generated `about.toml` does not accept (`release/notices.py:46-54` builds it from `deny.toml`, whose default allow list is MIT and Apache-2.0). The Linux x64 job ran out of disk, with 9 MB free, while `scripts/test_schema_release.py:113` compiled biodata into a fresh `/tmp/biomcp-schema-proof-*/target`.
- Starts from: `release/notices.py`, `deny.toml`, `scripts/test_schema_release.py` and the docs-check job in `.github/workflows/release.yml`.
- Rules: first find whether each crate ships in a release binary. A crate that ships only in dev or build dependencies leaves the notices, through cargo-about's own dependency-kind setting, with no license exception. A crate that ships gets a per-crate exception pinned to its exact version in `deny.toml`, which both cargo-deny and the notices read; MIT-0 and Zlib are approved permissive licenses for this purpose. Explain why the routine cargo-deny check passed and close that gap if it is a target or dependency-kind mismatch. The schema proof reuses the job's existing Cargo target directory instead of a fresh one, and the job prints free disk before and after the proof.
- Proof: notice generation passes for every release target triple locally; cargo-deny still passes; the schema proof's peak disk use in the docs-check job drops and is on the ticket; the next private rehearsal passes both steps.
- Changes: the notice generator or `deny.toml`, the schema proof test and the docs-check job.
- Defers: none.
