# 0478 — Compare recursive schemas in the compatibility check

Status: COMPLETE.
Landed: c558c080abe46bbee5bf1e0a46a46accfcaac601

Milestone: 5

## Outcome

The schema compatibility check compares schemas whose `$ref` graph has cycles, such as a Boolean expression tree whose AND, OR and NOT nodes contain nodes. It keeps every existing compatibility rule, and it still refuses unresolved, external and nonproductive cycles.

- Why: 0473's CIViC profile expression is a real recursive type, and the current check refuses any repeated reference, so no schema that carries a variant subject can be generated (m15043).
- Starts from: `compatibility.rs`, `compatibility_cases.rs` and the schema views in `reference.rs`.
- Rules: treat a pair of definitions already under comparison as an assumption that holds only if the rest of the comparison succeeds, never as a pass by itself. Keep every existing owner limit and add no new allowance. No API, JSON value or baseline change.
- Proof: focused cases for self recursion, mutual recursion, a rename that changes only the name, a narrowed cycle that must fail, a redirected or missing edge, an unresolved or external reference, and a nonproductive cycle; the existing compatibility cases still pass.
- Changes: the compatibility checker and its cases.
- Defers: nothing.
