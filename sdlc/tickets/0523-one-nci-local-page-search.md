# 0523 — Share one NCI local-page search

Status: COMPLETE.
Landed: 213a859b4c727e64bc60a30d7379eeb84e285223

Milestone: 5

Depends on: 0522

## Outcome

Organization search and biomarker search run one generic NCI fetch-once, page-locally search that takes the adapter function, namespace and "too broad" error.

- Why: Jarvis's 1.0 code review (m17823, finding 3). `organization.rs` and `biomarker/search.rs` share about 120 of 170 lines: the same NCI source, the same flow and the same key help.
- Starts from: the two search modules and the shared snapshot store after 0522.
- Rules: no reply, problem code or key help text changes.
- Proof: the case suite passes unchanged and the schema check passes, with the reply diff reviewed as a diagnostic; the existing organization and biomarker cases pass unchanged.
- Changes: the two search modules and one shared helper.
- Defers: nothing.
