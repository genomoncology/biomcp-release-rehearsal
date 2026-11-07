# 0765 — Label discovery case passes on every host

Status: OPEN.

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
