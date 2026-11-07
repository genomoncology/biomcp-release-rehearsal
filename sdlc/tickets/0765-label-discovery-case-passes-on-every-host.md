# 0765 — Label discovery case passes on every host

Status: COMPLETE.
Landed: acbca3baf23b1ab9098ac882b9e71a8a973dda83

Milestone: 5

## Outcome

`tests/cases/drug-label/drug/label-discovery.yaml` passes on main on every build host, or the product bug it shows is fixed.

## Why

Job j35187 at main bc6b467d8 on the M5 failed it: "step 1: /result/items/0 omits: unexpected field name". It is a recorded case that 0763 does not touch, and main's land gates passed it, so either the host's local data or the recording differs, or a real field leaks into the reply.

## Proof

- Proof: the case passes at the fixed commit on the M5 and on the default build host. If the cause is a product bug, the case catches it on main.

## Starts from

- Starts from: the case file, its recording, and what reads drug labels on the M5 that the default host lacks (a local copy, a config file or an environment variable).

## Keeps

- Keeps: the case's claims: free-text discovery without harmonized names, and exact application searches without product substitution.

## Defers

- Defers: nothing.

## Changes

- Changes: the case, its recording, or the drug-label code, as the cause shows.

## Depends on

- Depends on: nothing.

## Finding

- The M5 failure was not host state. Job j35187 ran `tests/tool-cases/drug/label-discovery.yaml`, a stale duplicate no gate runs. It expected no `name`, but label items have shown the SPL set ID as the name since 0605, and its later steps predate the free-text and controls cases. The main case `tests/cases/drug/label-discovery.yaml`, with `label-free-text.yaml` and `label-controls.yaml`, holds every claim but one and is the proof. The dropped claim is the `CAVHANZA EXTRA` step: its absent, exit 1 form has been stale since 0419, which made such searches complete with no items, and no gate held it. The tool-case copy is deleted.
