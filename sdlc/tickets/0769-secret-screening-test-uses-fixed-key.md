# 0769 — Secret screening test uses a fixed key

Status: OPEN.

Milestone: 5

## Outcome

`release.test_security.Screening.test_alphagenome_public_literals_pass_but_fresh_credentials_fail` gives the same result on every run and every host, because its synthetic credential is fixed and the pinned scanner always flags it.

## Why

Practice release run 42441596246 at 42b0ab1ea failed in the Linux x86_64 `qualify` job, docs-check stage: "AssertionError: 0 != 1 : artifact screening boundary changed". Every other stage and platform passed, and the same test passed on the M5 at the same commit (j35495). The test builds its credential with `secrets.token_hex(24)`. With the pinned gitleaks 8.30.1, a random 48-character hex value after `api_key = ` was missed in 2 of 200 tries, so the test fails about once in a hundred runs. A timing or chance failure is fixed, not rerun.

## Proof

- Proof: the test uses a credential that does not change between runs and is not a literal secret in source (for example derived from a fixed seed), and it still asserts exit 1 and the `generic-api-key` rule. Running the test's scan step 50 times in one job gives the same result each time.

## Starts from

- Starts from: `release/test_security.py` lines 38-56.

## Keeps

- Keeps: the scanner pin, its config, the public-literal half of the test, and the check that the scanner output never discloses the synthetic value.

## Defers

- Defers: nothing.

## Changes

- Changes: `release/test_security.py` only. Check other tests that pass a fresh random credential to the scanner and fix them the same way.

## Depends on

- Depends on: nothing.
