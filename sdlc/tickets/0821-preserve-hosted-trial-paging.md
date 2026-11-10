# 0821 — Preserve trial continuations across hosted tasks

Status: COMPLETE.
Landed: bedfaf5351a3ae7d3eed68a10c374705c3c1b201

Milestone: 5

Depends on: 0814

## Outcome

A ClinicalTrials.gov continuation issued by one hosted BioMCP task works on another task with an independent local store. Validate its selectors, consumed offset, caller scope and required source policy before contacting the registry.

## Why

Trial paging now saves the registry cursor in a task-local store. The hosted service runs two separate cores behind a load balancer. A valid continuation can reach a core that cannot find the saved binding. Before 0814, trial paging forwarded the registry cursor without requiring the issuing task's store.

## Proof

- In a local two-task test with separate stores and a synthetic registry, obtain an emitted cursor from task A and follow it through task B using a fresh client. Assert the same registry cursor bytes, filters, matching mode and consumed offset. Repeat after A stops.
- Changed offset, selector, capability or caller scope and excluded dependencies refuse before provider access. Retain the existing valid paging and provider-rejection restart controls.

## Starts from

- Core `trial/continuation.rs` and its existing binding tests, source snapshot ownership and exclusions, and hosting `infra/lib/service.ts` and `infra/lib/edge.ts`. The separate task stores and lack of affinity are confirmed by code inspection.
- Establish the two-store witness first. Review the smallest portable design before changing runtime or hosting configuration. The current proposal uses a versioned authenticated and encrypted token with a dedicated shared deployment secret. Its caller and source checks must preserve existing authorization.

## Keeps

- Native cursor bytes, native offset counting, local CLI behavior, current access policy and explicit restart guidance. Query records remain task-local. Do not assume clients keep affinity cookies, expose a sealing key, reuse provider or OAuth credentials as a key, or accept unauthenticated bindings.

## Defers

- Any hosting resource or account change requires the concrete rollout approval. Preserve the actual source identity of existing test and rehearsal evidence.

## Changes

- Hosted continuation behavior and owning local two-task proof. Any core/hosting configuration change belongs in the reviewed implementation design.
