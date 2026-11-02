# Nightly classifier accepts discovery-driven input changes

Status: COMPLETE.
Landed: aadf71ad24fad74f4a3c8607ae4368582fa3f553
Milestone: 5

## Outcome

The nightly live check classifies a complete command whose dependent requests changed only because upstream discovery returned different IDs as churn, including when a dependent group becomes empty, while it still fails on an input that discovery did not produce.

- Why: nightly j25989 failed `article/recommendations/multi.yaml` and `single.yaml` though both commands completed with HTTP 200. Multi now consumes five discovered PMIDs and no DOI-only inputs, so its DOI search correctly disappears; single consumes three PMIDs and two DOI-only inputs. `scripts/nightly_live_check.py:191` rejects an empty dependent group and `:212` rejects replacement selections of unequal size without a strict-subset relation. The same evidence classified as churn on 11-01 and as failure on 11-02 (m20111). A false failure trains people to ignore the nightly.
- Starts from: `scripts/nightly_live_check.py` (`classify_case`), `scripts/test_nightly_live_check.py`, evidence on Yellow at `data/biomcp-redux/0068-scheduled/20261102T090107Z-odzt3h2e` and `20261101T090108Z-827yf6_p`.
- Rules: accept a replacement or empty dependent selection only when every dependent input equals an ID the live discovery returned and the input context is unchanged. Keep every other failure rule. Refresh no capture.
- Proof: unit tests built from the two retained witnesses classify as churn; a variant with one input not from discovery classifies as failure; the iCite proof that reuses `classify_case` still passes.
- Changes: the classifier and its tests only.
- Defers: nothing.
