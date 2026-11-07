# 0696 — Plugin skill ships expanded

Status: COMPLETE.
Landed: 888ed99c3f4d5bdf4ea1bd9eb399ea6068eba958

Milestone: 5

## Outcome

The plugin's skill guide ships with its front map expanded, with no raw `{{front-map}}` marker, and a client finds `./skills/`.

## Why

Row W30 of `records/2026-11-06-pre-release-review.md`: `skills/SKILL.md:6` is copied raw at `release/workflow.py:119-126`, so every plugin user gets the marker. Traced in code, not run in a client.

## Proof

- Proof: a failing assertion in 0640's plugin section on the built plugin, then the fix.

## Starts from

- Starts from: `release/workflow.py:119-126`, `skills/SKILL.md`.

## Keeps

- Keeps: plugin contents otherwise.

## Defers

- Defers: nothing.

## Changes

- Changes: `release/workflow.py`, its test.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

The plugin reaches users two ways: the release zip and the repository root, because `.claude-plugin/marketplace.json` uses `source: "./"`. Fixing only the zip leaves `{{front-map}}` raw for marketplace users.

Keep the marker in an HTML comment in `skills/SKILL.md`, with one plain line beside it pointing to `biomcp skill install`. `crates/biomcp/build.rs` and `release/workflow.py plugin()` expand it for the binary and the zip.

The Proof splits. A unit assertion in `release.test_workflow` shows the zip's skill holds no marker, and one native build with an unpacked plugin shows the skill loads. The five-platform staged check belongs to the plugin section in 0640.
