# The SnpEff search fallback never matches an uncorroborated record

Status: COMPLETE.
Landed: 3c788c1cfb4ccc62db48e267a8a6067017b6bc16
Milestone: 5

## Outcome

When gene and protein variant search uses the SnpEff fallback, a candidate is marked matched only when its reference allele is confirmed. That means either another allowed source annotates the same record, or the reference genome agrees with the record's reference base. Other candidates are left out of the matched results and counted as excluded, with the reason.

- Why: hosting j28186 on 20bc8a9 (m23157), restricted: `search variant --gene BRAF --protein V600E` returned two matched candidates. One was the canonical chr7:g.140453136A>T (c.1799T>A, clinvar:13961). The other was chr7:g.140453136C>T (c.1799G>A), but the GRCh37 reference base there is A, so the record is invalid. A live MyVariant check on 2026-11-03 shows that record carries only a SnpEff annotation and a VCF block, with no dbNSFP, ClinVar or dbSNP data. The dbNSFP query returns only the canonical record, so the defect appears only in the 0593 fallback. The hosting team's clinical reviewer will not approve the prompt with only a documentation warning.
- Starts from: the 0593 fallback in `crates/biomcp-types/src/query/variant_search.rs`, `crates/biomcp-query/src/variant/search.rs`, and the SnpEff match in `crates/biomcp-adapters/src/myvariant/search.rs`.
- Rules:
  - The rule is general and never names BRAF or a variant.
  - Corroboration may come only from sources the caller's use allows (for example ClinVar or dbSNP in MyVariant), or from an NCBI reference sequence lookup that is bounded and paced and runs only for uncorroborated candidates.
  - An excluded candidate never appears as matched. The reply counts exclusions with a fixed reason.
  - With dbNSFP enabled, the request and reply stay byte-identical to the 0593 build.
  - Allowed exception to the command freeze for the hosted launch.
- Proof: installed-binary cases with recorded replies. Under `restricted`, BRAF V600E returns only the canonical record as matched and counts one exclusion. A synthetic SnpEff-only record whose reference agrees is still matched, and one whose reference disagrees is excluded. With dbNSFP enabled, the reply equals 0593's.
- Changes: the fallback match, a reference check if chosen, recorded replies, and cases.
- Defers: nothing.
