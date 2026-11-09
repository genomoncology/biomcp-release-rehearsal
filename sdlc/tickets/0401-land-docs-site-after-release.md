# 0401 — Refresh the docs before the cut and land them in the release session

Status: OPEN.

Milestone: 5

Depends on: 0294

## Outcome

Refresh the site against the selected release code before the cut. In the same release session, immediately after successful 1.0 publication and public-main advancement, main takes the prepared site as one squashed or re-authored commit with no agent attribution. Keep the dependency on 0294 for landing; preparation does not wait for it. The landing gate runs the site build, so 1.0 users receive current docs. The site build copies `release/install.sh` to the site root, and a check fails when the built site lacks `/install.sh`. The build works without a proxy example config, now that NO_PROXY accepts IPv6 and CIDR entries.

- Why: the site's installer must remain available and its 1.0 install lines must refer to published artifacts, while 1.0 users must not reach 0.9 docs. The November 6 PM ruling on W33 of `records/2026-11-06-pre-release-review.md` requires preparation before the cut and landing in the release session.
- Starts from: the `docs/site` head at landing time, `release/install.sh`, the Makefile gate and the pinned tool manifest from 0391.
- Rules: the docs lead owns the branch content until landing; the team does not edit it before then. Node and the site's packages are pinned in the tool manifest. Resolve the README conflicts in favor of released behavior, keeping 0397's Develop paragraph and 0400's proxy sentence facts. Publishing the site stays Ian's decision.
- Proof: the gate's site build passes; the `/install.sh` check fails on a build without it; `make docs-check` passes on the merged tree.
- Changes: the squashed site commit, the gate target, the install.sh copy and its check.
- Defers: publishing the site, which needs Ian.

