# 0527 — Let the live check see iCite citation growth as source churn

Status: COMPLETE.
Landed: 9671ae5f9653868a6da05165d354b484db7f5b54

Milestone: 5

## Outcome

The iCite reply that lists a paper's citing and referenced PMIDs declares them as a discovery, so the nightly live check classifies new citations as source churn instead of a failure.

- Why: nightly j24344 (2026-11-01) failed on one case, `article/citations/citing.yaml`. Every request returned 200 and the reply status stayed complete. The cited paper gained two citing papers (645 to 647), so the last iCite batch grew from 45 to 47 PMIDs, and the batch boundaries shifted across three requests. `scripts/nightly_live_check.py` `dependent_change` found no discovery for namespace `pmid`, because `biomcp_adapters::icite::Raw` implements no `discovery()`, so it returned failure. The PubMed, LitVar, CTgov and Semantic Scholar replies already declare theirs. The other six drift cases that night classified as churn.
- Starts from: `crates/biomcp-adapters/src/icite/`, the `discovery` and `discoveries` methods on other raw replies (`pubmed/raw.rs:26`, `litvar/raw.rs:95`), `crates/biomcp-fetch/src/observe_context.rs` and the j24344 evidence in the scheduled nightly folder on Yellow.
- Rules: the discovery lists the PMIDs in reply order exactly as the command consumes them. The classifier script stays unchanged. A real removal or an unexplained plan change still fails. No reply bytes change.
- Proof: replaying the j24344 report through the classifier with the new provenance classifies `article/citations/citing.yaml` as churn. A trial with a citing list grown by two at a batch boundary classifies as churn, and the same trial with an unrelated added PMID fails. The next scheduled nightly passes or fails only on another cause.
- Changes: the iCite adapter and its observation trial.
- Defers: nothing.
