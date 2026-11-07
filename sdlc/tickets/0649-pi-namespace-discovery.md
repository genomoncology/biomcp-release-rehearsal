# Keep Pi namespace tool discovery usable alongside full server instructions

Status: COMPLETE.
Landed: f636524bbb3b3ef6b3fbd5d2875fec06d143d0a5

Milestone: 5

## Outcome

Assess the evidenced Pi code-mode discovery problem and make the smallest justified metadata or client-description change, if needed, while preserving BioMCP's full usage, licensing and provenance instructions. Record whether the supported fix belongs in client configuration, the smoke prompt, or server metadata. Ian requested this ticket in m26153; it does not gate 1.0.

## Why

Pi's first smoke attempt sliced its namespace JSON to 2,000 characters before reaching the registered tool names, then guessed the wrong dotted identifier. The protocol returned the full namespace; Pi itself did not truncate it.

## Proof

- Proof: use the retained handshake and pinned Pi namespace serializer to reproduce the metadata ordering offline, then show reliable discovery using the supported returned tool identifiers. Keep the distinction between a model-chosen slice and an actual client/protocol limit. Reuse the successful corrected smoke instead of buying another run merely to recreate it.

## Starts from

- Starts from: Pi 1.4.0 `execute.js:649-653` and MCP `tools.js:45-48`; the j30356 namespace measurement on Yellow; the corrected harness branch `ticket/0632-pi-codemode` at `9266ff73c80740c85c58e924703f7c0953b68935`; successful j30474. Measured instructions: 2,470 characters; namespace JSON: 4,706; tools key offset: 2,564; `get_trial` identifier offset: 4,333. Pi registers `mcp__typed_smoke__get_trial`; the model incorrectly requested `mcp__typed_smoke.get_trial`.

## Keeps

- Keeps: typed mode remains opt-in, canonical tool names and all reply/schema/policy contracts remain intact, and client-selected namespace aliases remain client metadata. Preserve the full instructions and report actual cost for any later explicitly needed model run under Ian's recorded budget decision.

## Defers

- Defers: until after the current release qualification; no product-instruction change is part of candidate `1bb97e64f`.

## Changes

- Changes: typed smoke/discovery metadata fixtures and the smallest justified client-description or server-metadata adjustment after assessment; avoid a general discovery framework.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

The 2026-11-06 amendment on 0293 puts this ticket before the final rehearsal, so the line saying it does not gate 1.0 no longer applies.

The 2,000 character limit was a slice the model chose, not a protocol limit, so no server change is expected. The corrected harness is commit 9266ff73c on `ticket/0632-pi-codemode` and is not on main. Land it, run the smoke once, and close with the result.
