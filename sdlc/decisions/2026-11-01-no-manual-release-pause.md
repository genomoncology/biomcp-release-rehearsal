# The release workflow has no manual approval pause

Date: 2026-11-01. Ruling: Ian, relayed by the coordinator (m19076, m19082).

Ian does not want a review or approval step inside the release workflow. The workflow runs from a release tag through qualification, publication, installs from every channel and promotion without a GitHub environment reviewer or any other manual pause. The `release-publication` environment stays only as the name that Trusted Publishing and channel credentials bind to, with no required reviewers.

Ian's approval still decides whether a release happens: nobody pushes a release tag to the public repository or runs the history append without his explicit approval. Automated gates stay: qualification, tests, schema comparison, secret scan, attestation, install checks and the rule that promotion moves latest only for the newest stable release after every install passes. The rehearsal's no-op publish guards stay.

This replaces the reviewer-gated publication environment that 0533 built and the 2026-11-01 rehearsal terms that required one.
