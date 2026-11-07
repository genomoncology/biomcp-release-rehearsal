# 0733 — Skill and reply text read true to the agent

Status: COMPLETE.
Landed: c3c8bf84b8c2922645bb9c60f33599ed77957c8c

Milestone: 5

## Outcome

The skill's use cases say nothing about test data, so an agent reading them has no reason to doubt real rows. A typed reply never points at `structuredContent` when the client gets text: a client that gets a summary line and JSON reads a line that describes what it got.

## Why

The bench takes the skill byte for byte, and two of its 20 questions involve CPIC. Use cases 10, 02 and 13 carry wording such as "authored synthetic CPIC rows" and "These authored API replies are synthetic", and the others close with "Recorded replies demonstrate the commands". The use cases show no replies, so the wording only misleads. Typed replies say "JSON reply is in structuredContent" in the text form too, where the JSON follows in the same text.

## Starts from

- Starts from: `skills/use-cases/*.md`, `crates/biomcp/src/mcp/typed.rs`, `crates/biomcp/src/mcp/typed/summary.rs`, `crates/biomcp/src/mcp/protocol.rs`.

## Keeps

- Keeps: the sentence telling agents to read status, problems and source terms, and that live replies can change. Facts about test data stay, moved to the test cases that use it. The `StructuredOnly` form keeps its line, where it is true. Tool names, refusals and the reply JSON don't change.

## Changes

- Changes: `skills/`, `crates/biomcp/src/mcp/`, `tests/`.

## Proof

- Proof: a typed call under each reply form, through the real server, shows the right line. A search of `skills/` finds no test-data wording. Any installed copy of the skill or generated doc is regenerated, not hand-edited.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Amendment, 2026-11-06

PM ruling: the bench's three refusals are intended, but `variant normalize` on bare HGVS gives the agent no route to a working form. The reply now names that route (`search variant` with gene and coding change, then `get variant`), and the `assertions` refusal's instruction reaches `next` as well. This amends Keeps for these two refusals only. Hosting does not rely on either, and no other refusal changes.
