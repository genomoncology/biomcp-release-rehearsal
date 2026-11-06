# 0401 — Land the docs site after the 1.0 release

Status: OPEN.

Milestone: 5

Depends on: 0294

## Outcome

After 1.0 is published, main takes the `docs/site` branch as one squashed or re-authored commit with no agent attribution. The landing gate then runs the site build, so docs stay in step with code. The site build copies `release/install.sh` to the site root, and a check fails when the built site lacks `/install.sh`. The build works without a proxy example config, now that NO_PROXY accepts IPv6 and CIDR entries.

- Why: the docs lead finished `docs/site` (head edbd9f7ae; Jarvis m12426, 2026-10-28). Publishing the new site over the current one without `install.sh` would break every install line, and the site's install lines describe 1.0, so the site lands only after 1.0 is published.
- Starts from: the `docs/site` head at landing time, `release/install.sh`, the Makefile gate and the pinned tool manifest from 0391.
- Rules: the docs lead owns the branch content until landing; the team does not edit it before then. Node and the site's packages are pinned in the tool manifest. Resolve the README conflicts in favor of released behavior, keeping 0397's Develop paragraph and 0400's proxy sentence facts. Publishing the site stays Ian's decision.
- Proof: the gate's site build passes; the `/install.sh` check fails on a build without it; `make docs-check` passes on the merged tree.
- Changes: the squashed site commit, the gate target, the install.sh copy and its check.
- Defers: publishing the site, which needs Ian.

## Amendment, 2026-11-06

PM ruling (W33 of `records/2026-11-06-pre-release-review.md`): refresh the site branch against the release code before the cut, and land it in the release session, so 1.0 users do not reach 0.9 docs. This ticket's dependency on 0294 stays for the landing; the refresh does not wait for it. Publishing the site stays Ian's decision.
