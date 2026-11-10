# 0807 — Test existing channel credentials without publishing

Status: OPEN.

Milestone: 5

## Outcome

Run narrowly scoped GitHub Actions checks in the actual public BioMCP repository using its existing Homebrew secret and generated GitHub token. Establish what access those credentials have without publishing a release, image, formula or package, changing main, or exposing credentials. Retain actual outcomes and limits instead of asking Ian to attest to unknown token history.

## Why

Ian, November 10: "I haven't changed anything with the Homebrew tap token. I don't even know how you create it. I don't even know how that got created. How do I test any of this stuff? Can you just test it and try it?" Secret-name metadata alone cannot establish validity, and the local GitHub credential cannot read package permissions.

## Proof

- Check the existing Homebrew token through authenticated repository access and a git push --dry-run proposing only a local empty descendant of tap main. Verify no remote update; never run formulas or push that local commit. Report authentication and push-negotiation results accurately, including any remaining branch-policy limitation.
- Exercise GHCR push capability without publishing blobs/manifests/tags. A bounded empty-upload initiation followed immediately by cancellation is acceptable; send no payload bytes, validate cancellation location against the exact HTTPS registry/repository, attempt cleanup on every post-initiation path, and disclose any failed cleanup. Do not decode or log bearer credentials.
- Test guard/failure/redaction/cancellation behavior using controlled mocks, run focused helper/workflow checks, and obtain independent security review before any live execution. No native product build or release pipeline run is necessary.
- Record exact reviewed source/workflow hashes, actual run/ref/repository, safe status-only results and remote-state checks. Upload only explicitly named nonsecret reports, never credential files, environment dumps or the whole workspace.

## Starts from

- Existing registered public .github/workflows/release.yml supports normal release/rehearsal, not credential-only checks. HOMEBREW_TAP_TOKEN exists at repository scope; do not retrieve/copy its value. The token can be exercised inside this repository's Actions. Rehearsal repository has no copy and must not receive one.
- Build a tiny manual-only workflow/helper from reviewed private source, then prepare a temporary workflow-only branch parented ONLY by current public main. Use the registered workflow path for dispatch if a new workflow is not registered yet. Export only reviewed probe files; do not append private history, alter public default main/tags/release state or change account settings. Check inherited workflow triggers before any branch push.

## Keeps

- Existing credentials stay in GitHub's normal credential channel; never inspect, print, persist in evidence or transfer their values. Disable tracing and arbitrary response-body logging, reject cross-host authentication redirects, and scope all operations to the two intended channel repositories.
- Manual dispatch only, least job permissions, pinned existing actions, no publication commands or automatic retries. Existing tag/release approval and history-append procedure remain separate. Running candidate rehearsal stays unchanged and retains its own source identity.

## Defers

- Actual publication and full channel installation remain in the approved release session. If a nonpublishing probe cannot establish a permission conclusively, state that limit; do not silently perform a real write to tap main or publish an image to obtain proof.

## Changes

- Minimal manual credential-check workflow/helper and focused tests, plus retained execution evidence. No product runtime/schema/dependency or existing release-workflow redesign.
