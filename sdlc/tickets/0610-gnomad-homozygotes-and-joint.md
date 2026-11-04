# gnomAD population shows homozygotes and joint totals

Status: COMPLETE.
Landed: 16727e1a50c452dba843fea85333483c78bb27a8
Milestone: 5

## Outcome

`variant population` reports homozygote counts for exome and genome, and gnomAD's joint allele count, allele number and homozygote count, beside today's AC and AN.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 5. `variant population 1-11796321-G-A --detail` gives AC/AN and 305 group rows but no homozygotes and no joint totals; the gnomAD r4 API returns homozygotes of 80,805 (exome) and 6,918 (genome) and joint 513,548 / 1,613,846 / 87,723. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: `crates/biomcp-query/src/variant/population*.rs`, `crates/biomcp-adapters/src/gnomad/`, and `crates/biomcp-types/src/query/variant_population.rs` with its response type.
- Rules: the values come from gnomAD as published, never summed by BioMCP; joint totals appear only where gnomAD publishes them, and the release is named. If the current gnomAD request lacks the fields, extend that same request; no new source.
- Proof: an installed-binary case on the witness variant shows the five numbers above; a variant with no joint data says so; existing population replies change only by the added fields, named in the release notes.
- Changes: `crates/biomcp-adapters/src/gnomad/**`, `crates/biomcp-query/src/variant/population*.rs`, `crates/biomcp-types/src/query/variant_population.rs`, `crates/biomcp-types/schemas/variant-population*.json`, `tests/cases/variant/population*/**`, `tests/replies/variant-population/**`; the gnomAD request and mapping, the population reply and schema, cases.
- Defers: nothing.
