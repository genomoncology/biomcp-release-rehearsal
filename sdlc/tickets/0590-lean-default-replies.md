# Default replies drop repeated content

Status: COMPLETE.
Landed: 2d84360a21e6772fd6560851c869cad562f0ed3b
Milestone: 5

## Outcome

Default replies stop repeating what the agent already has: the article full-text index no longer repeats the card, part titles no longer repeat "Main text", and `next` suggests `article text --find` where it applies.

- Why: `planning/eval-baseline-2026-11-02.md` item 3 found bulky defaults, and the reply envelope is about 16% of JSON. Ian approved fixing the general causes before 1.0 (2026-11-03).
- Starts from: the article full-text view in `crates/biomcp/src/view/` and the typed next steps.
- Rules: only content repeated elsewhere in the same reply is removed; no fact is lost. Field selection and envelope trimming wait for experiment 513's ranked list.
- Proof: installed-binary cases show the smaller replies with every fact still present, and the byte change is recorded.
- Changes: the article views, next steps and their cases.
- Defers: field selection and envelope trimming, to the ranked list.
