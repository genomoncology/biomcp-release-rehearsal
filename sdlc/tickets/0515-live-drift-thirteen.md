# 0515 — Resolve the thirteen live drift failures

Status: COMPLETE.
Landed: 0af45c8eeefae00bdef80fd965600f34e9dd17fb

Milestone: 5

## Outcome

Each of the thirteen cases the 2026-10-31 nightly classified as a drift failure is resolved: either the adapter reads the source's current shape and the case's recorded replies are refreshed from a bounded public capture, or the case is shown to fail for a reason outside BioMCP and the classifier gains that exact reason. The next nightly reports zero failures for them.

- Why: the offline classification of nightly j24031 (m17566): 84 cases, 68 healthy, 3 churn, 13 failures, 0 errors. Failures: adverse-event/native-search.yaml; article/filters/include-retracted.yaml; article/litsense/licensed-subset.yaml; article/recommendations/multi.yaml and single.yaml; drug/label-discovery.yaml; gene/retired.yaml; pathway/examples.yaml, markdown.yaml and summary.yaml; pathway-sections/all.yaml and examples.yaml; trial/name-fallback.yaml.
- Starts from: the classified report on Yellow under biomcp-redux/0514-offline-classification, the cases and their adapters.
- Rules (m17620): synthetic label steps, the gene retired fixture and the LitSense licensed subset stay recorded-only controls and leave the nightly, whose coverage uses actual public source identities; LitSense stays uncalled beyond the one headers-only probe. The shared shape observation in observe.rs and live.rs reports incomplete sampling as incomplete, never as a removed field, with caps and diagnostics kept, and no classifier broadening. One bounded public keyless capture per affected source, no licensed content, KEGG stays uncalled; a field the source removed becomes absent with its existing problem, never a guessed value; group cases by source and fix each source once.
- Proof: per source, the refreshed case passes over its new recorded replies and the adapter change fails on the base build where one was needed; the next scheduled nightly classifies these cases healthy or churn.
- Changes: affected adapters, recorded replies and cases.
- Defers: nothing.
