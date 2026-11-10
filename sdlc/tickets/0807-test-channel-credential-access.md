# 0807 — Test existing channel credentials without publishing

Status: OPEN.

Milestone: 5

## Outcome

Exercise the existing Homebrew secret and generated GitHub token in the actual public BioMCP repository, and retain observed access results and limitations. This is external operator work: the one-off probe remains archived rather than being shipped as a production feature. Preserve failed cleanup reports alongside successful authentication/admission evidence.

## Why

Ian, November 10: "I haven't changed anything with the Homebrew tap token. I don't even know how you create it. I don't even know how that got created. How do I test any of this stuff? Can you just test it and try it?" Secret-name metadata alone cannot establish validity, and the local GitHub credential cannot read package permissions.

## Proof

- Both actual runs, 42884420606 and 42885938241, accepted Homebrew authentication and git push --dry-run negotiation while all tap refs remained unchanged. This does not prove acceptance by server update hooks or a real formula publication.
- Both GHCR checks returned token HTTP 200 and empty-upload HTTP 202, establishing upload admission. Both overall runs failed because the helper rejected the provider cancellation path; no DELETE was attempted. Two empty sessions have unconfirmed cleanup. No payload, blob, manifest or tag was sent. Do not report either overall run as passed.
- Preserve the original reviewed source 3dff32e46 and corrected source 9d13b5ea5, mocked proofs, security reviews, and exact public-export hashes. The generic relative-base bug was fixed, but the second live safe diagnostics prove its provider path did not match the helper's assumed prefix; the actual raw URLs were never retained or exposed.
- Receiver-verified live packets: Yellow data/biomcp-redux-evidence/0807/run-42884420606-65d5cfdb52f3 (manifest29e89ab196b3a9f3cd77b1a89c1f7c77043c311bd4040a3dbbb71a01d396b432) and run-42885938241-16fc4fe6b52a (manifest46d5740b620e0e4cda8ca6c83e1c6dc0b48c67713ed46f392b82e3a55a722838). Preserve safe reports, source/run identity and remote-state checks.
- Archive both actual public probe commits and remove only the two owned temporary branches after verifying their unchanged heads and retained evidence. Preserve Actions runs/artifacts and public main.

## Starts from

- The public probes were each one workflow-only child of public main561d3fe06d9dff5485f13a82a4108371f6fdfe88, using the registered release.yml path and existing repository secret without a deployment environment. Inherited workflows did not run on branch push.
- First public source65d5cfdb52f3a146e3993973c1f32b9124643f8d; corrected public source16fc4fe6b52a9d1ce1086fad27586dbb64628a0f. Neither appended private history or changed public main, tags, packages or account settings.

## Keeps

- Existing credentials stay in GitHub's normal credential channel; never inspect, print, persist in evidence or transfer their values. Disable tracing and arbitrary response-body logging, reject cross-host authentication redirects, and scope all operations to the two intended channel repositories.
- Existing tag/release approval and history-append procedure remain separate. Running candidate rehearsal stays unchanged and retains its own source identity. Stop further live probes; the requested access facts are established and cancellation remains an explicit limitation.

## Defers

- Actual publication and full channel installation remain in the approved release session. Shipping or reusing the archived helper requires the opaque-cancellation follow-up recorded in idea0810 before any new live session. This helper limitation does not establish a release-runtime or credential-permission defect.

## Changes

- Outcome record and external evidence references only. Keep prototype workflow/helper/tests in verified archive refs and bundles; do not land them into the product. No runtime/schema/dependency or existing release-workflow redesign.
