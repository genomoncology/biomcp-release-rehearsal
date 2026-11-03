# Sources carry their actual terms, and hosted callers declare one of three uses

Status: COMPLETE.
Landed: 2f109f95c3b44748fd6528e2f72ef32fcdb717d1
Milestone: 5

## Outcome

Every source and component carries the use label its published terms support, so nothing free for commercial use is blocked, and hosted callers declare `restricted`, `commercial` or `academic`.

Labels: `open` and `attribution` as today; `noncommercial` for terms that bar commercial use; `licensed` for sources that need a separate license even when a key is sent; `per-record` for full text whose rights come from each record's own license. The catch-all `restricted` label is retired, because `restricted` becomes a use name.

Uses:
- `restricted`: only `open`, `attribution` and the commercial-permitted records of `per-record` sources. A key never enables a `noncommercial` or `licensed` source, but it still raises limits on a free source such as NCBI or openFDA. A hosted request without the header gets this use.
- `commercial`: the `restricted` set plus any source the caller sends a key for, the key being their license.
- `academic`: every configured source.

- Why: Ian's ruling of 2026-11-03, relayed by the hosting team in m22678: "There shouldn't be any limits to anything that's free for commercial use", with three levels (restricted, commercial, academic), and restricted for hosting "so I'm just going to make it very restricted for now". Today hosted commercial calls disable every `Use::Restricted` source (`crates/biomcp/src/command/hosted_use.rs`). That label means "mixed or unreviewed rights", so commercial callers lose ClinicalTrials.gov, PubMed, ClinVar, openFDA, MyGene, MyVariant, GWAS Catalog, cBioPortal, Europe PMC and more.
- Starts from: `Use` in `crates/biomcp-types/src/source.rs`, the 29 `Use::Restricted` and 8 `Use::Noncommercial` declarations under `crates/biomcp-adapters/src`, `DeclaredUse` and `restrict` in `hosted_use.rs`, and the header parse in `crates/biomcp/src/mcp/http.rs` and `http/wire.rs`. The hosting team holds the verbatim terms for all 35 declarations, fetched 2026-11-03 (m22678).
- Rules:
  - Labels follow the m22678 summary. `open` or `attribution`: ClinicalTrials.gov, ClinVar, PubMed, PubTator3, LitSense (licensed subset still uncalled), NCBI E-utilities, openFDA, ClinGen Allele Registry and eRepo, GWAS Catalog, cBioPortal, Cancer Hotspots, MyGene.info, the MyVariant service, SnpEff, Mutalyzer, VariantValidator, CPIC (split from ClinPGx), NCI CTS (key required, still free). `per-record`: PMC OA, Europe PMC full text, Semantic Scholar PDFs. `noncommercial`: DDInter, ClinPGx data, the Semantic Scholar API, ORCID, WHO PQ and IVD, and the MyVariant components CADD, the dbNSFP academic scores and SNPedia. `licensed`: OncoKB, OMIM, AlphaGenome, and the COSMIC component in MyVariant.
  - FAERS, MAUDE and VAERS are `attribution`, so `restricted` serves them. Ian, 2026-11-03 (m22685): "Yeah, I definitely want to turn those on." The MedDRA and GMDN terms come inside CC0 openFDA data, and those dictionaries' licenses bar redistributing the dictionary or building categorization services, not showing reports. BioMCP's VAERS counts meet CDC WONDER's statistical-use terms.
  - Fix the stale terms links: CADD (https://cadd.gs.washington.edu/download), COSMIC (https://www.cosmickb.org/terms/), SnpEff (MIT).
  - Local CLI and stdio behaviour is unchanged: with no declared use, they run every configured source as today. The `x-biomcp-use` header is read only in hosted mode.
  - `restricted` stays a legacy `disable_sources` alias for exactly the former group, so existing local configurations and help keep working (m22690).
  - Other named `disable_sources` groups such as `noncommercial` follow the corrected labels, and the release notes name each membership change (m22713).
  - Allowed exception to the command freeze, because it serves the hosted launch: label text in source listings, the usage and terms fields of source outcomes, the hosted header values and their error text may change. Every other reply, help and skill stays byte-identical, and the reply diff names each change.
  - Keys never appear in replies or logs.
- Proof: installed-binary hosted cases. Without the header, a ClinicalTrials.gov, PubMed and ClinVar call succeeds, while an OncoKB call with a valid-looking key and a DDInter call are refused as disabled. With `commercial` and an OncoKB key, OncoKB is attempted; with `commercial` and no key, it is refused. With `academic`, every configured source is attempted. A CC BY full-text record is returned under `restricted` and a noncommercial-licensed one is withheld. An unknown header value names all three values. A local CLI run with no header matches today's replies.
- Changes: `crates/biomcp-types` (`Use`), the adapter declarations, `hosted_use.rs`, the hosted header parse and error, the schema baseline for the `Use` enum, and the cases above.
- Defers: nothing.
