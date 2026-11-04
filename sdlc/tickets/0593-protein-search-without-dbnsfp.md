# Gene and protein variant search works when dbNSFP is off

Status: COMPLETE.
Landed: 4d30e2e9b2c9db13b37bf3cb7e6ce3252a1f547f
Milestone: 5

## Outcome

`search variant --gene G --protein P` finds MyVariant candidates through SnpEff's open gene and protein fields whenever the dbNSFP component is disabled, as it is under the hosted `restricted` use. With dbNSFP enabled, the query and reply stay byte-identical to today.

- Why: hosting j28029 on d1f85dd47 (m22946), a restricted hosted call: `search variant --gene BRAF --protein V600E` returned a tool error with MyVariant skipped as disabled. The declared query selects on `dbnsfp.genename` and `dbnsfp.hgvsp` (`VariantSearch::source_query` in `crates/biomcp-types/src/query/variant_search.rs`), and `crates/biomcp-query/src/variant/search.rs` refuses the page when the `dbnsfp` component is disabled (0585 made it `noncommercial`). So restricted callers cannot search variants by gene and protein at all. This blocks the hosted Amazon tests this week.
- Starts from: `source_query` and `uses_dbnsfp_selector` in `variant_search.rs`, `component_terms` in `biomcp-query/src/variant/search.rs`, `components.rs` and the SnpEff transcript match in `crates/biomcp-adapters/src/myvariant/search.rs`.
- Rules:
  - When `dbnsfp` is disabled and `snpeff` is enabled, select on SnpEff's annotation gene name and `hgvs_p` fields, matching the protein forms `matches_protein` already accepts, and say in the reply which selector was used.
  - When both are disabled, keep today's named refusal.
  - A raw `--query` stays refused while any component is disabled.
  - No dbNSFP, CADD, COSMIC or SNPedia field appears in the request or the reply under `restricted`.
  - Allowed exception to the command freeze for the hosted launch: only replies with dbNSFP disabled change.
- Proof: installed-binary cases with recorded MyVariant replies. With `--disable-sources noncommercial`, and in a hosted `restricted` call, BRAF V600E returns candidates whose request names only SnpEff fields. With every component enabled, the request and reply equal the 0591 build.
- Changes: `variant_search.rs`, `biomcp-query/src/variant/search.rs`, recorded replies, and cases.
- Defers: nothing.
