# Biodata variants and annotations: a common model

This plan sets one model for variants and annotations that BioMCP's `biodata` crate and another biodata consumer share until the two crates merge. The other consumer projects HGVS between genomic, transcript and protein forms. BioMCP adapts public sources for agents. Each team builds its own crate under `decisions/2026-10-24-biodata-builds-its-own-parts.md`. Copied code is allowed. Both crates follow the names, wire spellings and rules below, so the later merge is a move and not a rewrite.

The plan covers three crates:

- **BioMCP biodata**, this repo's `crates/biodata`.
- **The projecting crate**, the other consumer's `biodata`. It holds the HGVS grammar and its parsed form, alteration and cytogenetic notation, coordinates and an allele key. Its sibling crate holds normalization, projection and the annotation readers.
- **The standalone crate**, the public `biodata` repository.

## 1. What each crate has today

| Topic | BioMCP biodata | Projecting crate | Standalone crate |
| --- | --- | --- | --- |
| One-based position | `Position1(NonZeroU32)`, serializes as an integer ([coord.rs](../../crates/biodata/src/coord.rs)) | `Position1(u32)` with zero rejected and no serde | None |
| Zero-based span | `Span0 { start, end }` as u32, half-open, empty allowed, wire `{"start","end"}` ([coord.rs](../../crates/biodata/src/coord.rs)) | `Span0 { start, end }` as u32, half-open, empty allowed, no serde | `GenomicInterval { start, end }` as u64, half-open, capped at 2^53 − 1, with a constant label `zero_based_half_open` in its schema |
| Region on a chromosome | `GenomicInterval { build?, chromosome, span, strand? }` ([coord/genomic.rs](../../crates/biodata/src/coord/genomic.rs)) | `GenomicInterval { build, contig, span }`, no strand, no serde | `GenomicLocus { assembly, sequence, interval, strand? }` |
| Build | `Build`: Rust cases `Grch37` and `Grch38`, wire `GRCh37` and `GRCh38`; an absent build is `Option<Build>` | `Build`: `GRCh37`, `GRCh38`, with no unstated case and no hg19 alias | No enum. Assembly is free text with an authority |
| Chromosome | `Chromosome(u8)`: 1 to 22, X, Y, MT, canonical spellings only; `Contig(Box<str>)` keeps a source name outside that set | `Contig(u8)`: 1 to 22, X, Y, MT; reads an optional `chr`; writes `7`, `X`, `MT`; maps to the `NC_` accession per build from a fixed table | `sequence: String`, kept as given |
| Strand | Rust cases `Forward` and `Reverse`, wire `+` and `-` | `Plus`, `Minus` on the wire | `+`, `-`, `.`, `?` |
| Sequence accession | `Accession` copied from the projecting crate for variant values ([variant/accession.rs](../../crates/biodata/src/variant/accession.rs)); gene values still use `RefSeqTranscript` and `EnsemblTranscript` with an optional version, and `ManeSelect` requires a version ([id/transcript.rs](../../crates/biodata/src/id/transcript.rs)) | `Accession`: RefSeq (`NC_`, `NG_`, `NM_`, `NP_`, `NR_`, `NT_`, `NW_`, `XM_`, `XP_`, `XR_`), Ensembl (`ENSG`, `ENST`, `ENSP`) or LRG, with a required version and exact digit widths | Opaque text |
| Bases | `Base` IUPAC DNA enum and `Sequence`, which may be empty ([variant/sequence.rs](../../crates/biodata/src/variant/sequence.rs)) | `Base` enum of the IUPAC codes plus U; `Bases` is nonempty | Uppercase IUPAC text in `SequenceReplacement` |
| Allele | `SequenceLocation { sequence, span }`, `Allele { location, reference, alternate }` and `Representation { source, expression, build?, allele? }` ([variant/allele.rs](../../crates/biodata/src/variant/allele.rs), [variant/model.rs](../../crates/biodata/src/variant/model.rs)) | `AlleleKey { interval, reference, alternate }`, where the reference length equals the span length. It is built only after normalization, at the rightmost placement | `SequenceReplacement { reference_id, interval, reference_state, alternate_state }`, nonempty, with the alternate required to differ from the reference |
| HGVS | `HgvsText` checks the published form and keeps the text unchanged; `Expression` checks each syntax ([variant/notation.rs](../../crates/biodata/src/variant/notation.rs)). A public query type `ProteinChange` serves article and variant search | Full winnow grammar with a parsed form: `Hgvs { predicted, reference?, change }`, nucleotide and protein edits, typed residues and a `Parsed { source, outcome }` envelope. It refuses alleles, mosaics, repeats and fusions as unsupported | Hand-written scanners for HGVS 21.1.4, with digits kept as text and no serde on the parsed form |
| Variant identifiers | `VariantId` enum over raw values: `Rs`, `Ca`, `ClinVarVariation`, `ClinVarAllele`, `ClinVarAccession` and `Vrs` ([variant/id.rs](../../crates/biodata/src/variant/id.rs)); `Variant { ids, representations, genes: [GeneSymbol] }` | None in the crate. The readers hold rsID, CAid and ClinVar accessions as `String` and the Variation ID as `u32` | Opaque text; ClinVar Variation ID as `u64` |
| Annotations | Gene level: `OncokbCard`, `ClinGenValidity` and `CivicGene` with source labels as `Text` ([response/gene.rs](../../crates/biomcp-types/src/response/gene.rs)). Variant sections have landed; shared `Statement` and `PopulationFrequency` values live in biodata, and consumer response types retain source records ([statements](../../crates/biodata/src/variant/statement/mod.rs), [population frequencies](../../crates/biodata/src/variant/population.rs), [variant responses](../../crates/biomcp-types/src/response/variant.rs)) | In the reader crate: ClinVar records with per-submission classification kind (germline, somatic, oncogenicity, none), review status and origin; ClinGen interpretations with criteria codes; hit records with `Provenance { source, release, row_id }` and a match relation. Every label is publisher text | Adapter projections for ClinVar, MyVariant, OncoKB, cancer hotspots and gnomAD, with labels as text |
| Clinical frameworks | No enum; labels are text | No enum; labels are text | No enum, by its scope rule |
| Provenance | `Source` enum with short names; `Section` names its source; `SourceRecord { id, url? }` ([response/section.rs](../../crates/biomcp-types/src/response/section.rs)) | Source family as text, release as read from the data file, publisher row ID | `source`, `locator`, `source_release` on each value |
| Option meaning | The source has no value | The publisher did not supply it | The source did not state it |
| Enums | `canonical_enum!` and `wire_enum!` declare each spelling once; snake_case; unknown values become a problem and leave the field absent ([macros.rs](../../crates/biodata/src/macros.rs)) | snake_case in the crate; PascalCase on several reader enums; refusal codes for unknown input | Per-type spellings |
| Serde | Both directions, through the same checks; internally tagged enums; `deny_unknown_fields`; no flatten | Serialize behind a feature, some Deserialize; externally tagged enums; flatten in writers | No derives on core types; hand-written documents |
| Schemas | `schemars`, checked in under `schemas/`, named `biodata.<entity>/<major>`, with drift and compatibility tests ([tests/schemas.rs](../../crates/biomcp-types/tests/schemas.rs)) | None | Hand-written JSON Schema 2020-12, unversioned by rule |
| Error codes | Dotted stable codes, mostly two parts, such as `build.unknown` and `position.zero` ([error.rs](../../crates/biodata/src/error.rs)) | Dotted stable codes such as `coordinate.zero_position` and `hgvs.edit.invalid` | Flat snake codes such as `hgvs_routing_unrecognized_prefix` |
| Toolchain | Edition 2024, Rust 1.95.0; serde, winnow and optional schemars | Edition 2024, Rust 1.95.0; winnow and optional serde | Edition 2024, Rust 1.86; serde, serde_json, XML readers, sha2 |

## 2. Agreement, conflict and merge risk

### Where the crates already agree

- Coordinates name their counting rule in the type. All three use zero-based half-open spans as the stored form, and the two working crates share the names `Position1` and `Span0` with the same meaning.
- Build spellings are `GRCh37` and `GRCh38`. Neither crate infers a build.
- No crate does I/O. Each `clippy.toml` bans file, network, process and clock access.
- Every crate keeps source labels as given and never classifies a variant itself.
- `Option` means the source has no value.
- Unknown input is refused with a stable code that never holds the input text.
- Each crate keeps the input text beside any parsed or resolved result.

### Where they conflict

1. **One name, three meanings.** `GenomicInterval` means a strand-bearing chromosome region here, a strandless region in the projecting crate, and bare bounds in the standalone crate. `Contig` means a kept source spelling here and a checked human chromosome number there. `ProteinChange` names a search query here and the parsed HGVS protein change there.
2. **Strand and build names.** `+` and `-` on the wire here and in the standalone crate; `Plus` and `Minus` in the projecting crate. Rust case names differ in both enums.
3. **Absent build.** This crate makes the build optional on `GenomicInterval`. The projecting crate requires it.
4. **Accessions.** This crate's gene values allow unversioned transcripts and accept six or more digits. The projecting crate requires a version and an exact digit width. The standalone crate keeps text.
5. **Integer width.** u32 in the working crates; u64 in the standalone crate.
6. **Wire contract.** This crate checks both directions and versions every schema. The projecting crate mostly serializes and has no schema. The standalone crate writes documents and schemas by hand and versions nothing.
7. **Enum tagging.** Internally tagged here; externally tagged in the projecting crate's HGVS form.
8. **Annotation shape.** BioMCP now has shared `Statement` and `PopulationFrequency` values alongside source records with free-text labels. The other crates keep their own records per source without a shared statement type.
9. **Source identity.** An enum here; text in both others.
10. **Error code style.** Dotted in the working crates; flat snake in the standalone crate.

### Conflicts that would make a merge hard

Conflicts 1, 2, 3, 4 and 6 change public JSON or public Rust names, so each one costs a schema major version or a breaking release if both crates ship first. Conflict 8 grows with every source either team adds. These are settled by section 3, the rules in section 4 and the list in section 6. Conflicts 5, 7 and 9 are local and can be settled at the merge. Conflict 10 is settled by the shared refusal codes in section 3.11.

## 3. The common model

The model follows GA4GH VRS 2.x for alleles, GA4GH Cat-VRS 1.x for protein-level and compound subjects, and GA4GH VA-Spec 1.x for statements. It follows HGVS 21.1 for notation and NCBI SPDI for the interbase allele form. It keeps the existing crates' conventions for Rust names, snake_case JSON, checked newtypes and versioned schemas. Ian chose snake_case as the native wire spelling (section 5).

### 3.1 Sequence references and coordinates

| Type | Fields and wire form | Rule | Standard |
| --- | --- | --- | --- |
| `Accession` | Canonical text, such as `"NC_000007.13"`, `"NM_004333.4"`, `"NP_004324.2"`, `"ENST00000288602.11"`, `"LRG_199p1"` | The only versioned accession grammar. Version required, written without leading zeros, so `.04` is refused. RefSeq numbers have exactly six or nine digits, Ensembl numbers eleven. Human Ensembl prefixes only. Parse and display round-trip. The projecting crate's grammar is the one copied. `RefSeqTranscript` and `EnsemblTranscript` fold into `Accession` and `AccessionBase` | HGVS reference sequence rules; RefSeq and Ensembl formats |
| `AccessionBase` | Canonical text without a version, such as `"NM_004333"` or `"ENST00000288602"` | The unversioned twin of `Accession`, with the same prefixes and digit widths, for sources that give no version. Variant, allele and MANE values never use it | Same |
| `Build` | `"GRCh37"`, `"GRCh38"` | Rust cases `GRCh37` and `GRCh38`. `#[non_exhaustive]`. No `unstated` case: an absent build is `Option<Build>` omitted from JSON. No hg19 or hg38 alias on the wire; adapters map provider spellings | GRC names |
| `Chromosome` | `"1"` to `"22"`, `"X"`, `"Y"`, `"MT"` | The checked human primary set, never with `chr`. Adapters read `chr7`, `chrM` and `23` and map them. This is the projecting crate's `Contig(u8)`, renamed | GRC primary assembly |
| `Contig` | Source spelling text | The source's own name, kept only where a source gives a name outside the primary set or where the source spelling matters | None |
| `Strand` | `"+"`, `"-"` | Rust cases `Plus` and `Minus`. Absence is `Option<Strand>` | GFF3 and BED |
| `Position1` | Positive integer | One-based, as today. Also the type of every protein residue position | HGVS, VCF |
| `Span0` | `{"start": n, "end": n}` | Zero-based half-open, empty allowed, as today | VRS interbase, SPDI |
| `SequenceLocation` | `{"sequence": Accession, "span": Span0}` | A location on one versioned sequence. This is the variant location type. On a protein accession it is a protein location | VRS `SequenceLocation` with an accession in place of a refget digest |
| `GenomicInterval` | `{"build"?, "chromosome", "span", "strand"?}` | A region labeled by chromosome and the source's stated build, for gene spans and region search. Build and strand are optional. Containment and overlap refuse when either interval has no build. A gene extent is a `GeneRef` beside a `GenomicInterval`, with no separate type | None |

The pure map between (`Build`, `Chromosome`) and the `NC_` accession is a fixed table with no I/O. The projecting crate owns it, and BioMCP copies it.

### 3.2 Alleles

| Type | Fields and wire form | Rule | Standard |
| --- | --- | --- | --- |
| `Base` | One uppercase IUPAC DNA letter | DNA only. `U` is refused. RNA notation stays in `HgvsText`, and an RNA base type lives in the projecting crate's grammar | IUPAC |
| `Sequence` | A string of bases, such as `"A"`, `"GA"` or `""` | May be empty | VRS literal sequence, SPDI |
| `Allele` | `{"location": SequenceLocation, "reference": Sequence, "alternate": Sequence}` | Interbase and unpadded. The reference length equals the span length. Both sides empty is refused. A reference equal to the alternate is allowed and states the reference allele. The constructor never trims, shifts or checks the reference genome | SPDI `S:P:D:I`; VRS `Allele` with the deleted sequence kept |
| `AlleleKey` | `{"build": Build, "allele": Allele}` | Only the projecting crate's normalizer builds it. It holds the normalized placement and the build, because an accession does not always name its build: `NC_012920.1` is the mitochondrial sequence in both GRCh37 and GRCh38. It is the join key for that crate's annotations | The projecting crate's existing rule |

`Allele::trimmed()` is allowed as a pure function that removes a shared prefix and suffix and shifts the span to match. It never reads a reference sequence. A placement type beside `Allele` is rejected.

Equality of two `Allele` values is equality of what the sources stated. It is not variant equivalence. Equivalence needs normalization against the reference sequence, which only the projecting crate does. A VCF row converts to an `Allele` by dropping the padding base and subtracting one from the start. A gnomAD `chrom-pos-ref-alt` ID converts the same way.

### 3.3 Notation and representations

| Type | Fields and wire form | Rule | Standard |
| --- | --- | --- | --- |
| `Hgvs` (text) | Canonical text, kept exactly as given, such as `"NM_004333.4:c.1799T>A"` | Checked only for the published form: an `Accession`, an optional accession or gene symbol in parentheses, such as `NM_004333.4(BRAF):c.1799T>A`, a colon, one of `g.`, `c.`, `n.`, `r.`, `p.`, `m.`, `o.`, and a nonempty printable body. Text with no reference is refused. Name in Rust: `HgvsText` | HGVS 21.1 |
| `HgvsParsed` | The projecting crate's parsed form | Built only by the projecting crate's grammar. It never replaces the text: any value that holds a parsed form also holds the `HgvsText` it came from. Its JSON is not a public contract until the merge | HGVS 21.1 grammar |
| `ExpressionSyntax` | `"hgvs.g"`, `"hgvs.c"`, `"hgvs.n"`, `"hgvs.r"`, `"hgvs.p"`, `"hgvs.m"`, `"hgvs.o"`, `"spdi"`, `"gnomad"`, `"vcf"`, `"iscn"` | VRS spellings, plus `vcf` for `chrom:pos:ref:alt` text and `hgvs.o` for HGVS circular genomic text, which `HgvsText` accepts. A later GA4GH export leaves out `hgvs.o` and `vcf` expressions, which VRS does not list | VRS `Expression.syntax` |
| `Expression` | `{"syntax", "value", "syntax_version"?}` | The value is checked against its syntax's published form at construction | VRS `Expression` |
| `Representation` | `{"source", "expression", "build"?, "allele"?}` | One source's own form of a variant. `allele` is present only when the source gave reference, alternate and interbase bounds, or a padded VCF row the adapter converted under section 3.2. `build` is the source's stated build | VRS `expressions`, ClinGen Allele Registry genomic and transcript alleles |

A MyVariant `_id` such as `chr7:g.140453136A>T` is not reference-qualified HGVS. It becomes a `Representation` with syntax `hgvs.g`, the value as given and `build` set to GRCh37 only when the request named that build. A ClinGen Allele Registry allele becomes one `Representation` per genomic and transcript allele, each with its `allele` and its stated build.

### 3.4 Variant identifiers

Each identifier is its own newtype: `RsId`, `CaId`, `PaId`, `VcvId`, `RcvId`, `ScvId`, `ClinVarVariationId`, `ClinVarAlleleId`, `CosmicId` and `VrsId`. `VariantId` is the enum over the variant-level newtypes only, each case wrapping its newtype. Each case serializes as its published text, and parsing picks the case from that text alone. Each case names a different level of thing, so a join on one ID is a join at that level.

| Case | Newtype and wire text | Check | Level | Standard |
| --- | --- | --- | --- | --- |
| `Rs` | `RsId`, `"rs113488022"` | `rs` and a positive number, stored as u64 | A dbSNP site, which may carry several alternate alleles. Never an allele join key | dbSNP |
| `Ca` | `CaId`, `"CA123643"`, `"CA000434"` | `CA` and a positive number written with at least six digits, zero-padded to six as the registry publishes it. Text with fewer than six digits, or more than six digits starting with zero, is refused. It serializes as that canonical text and compares by its number | One nucleotide allele | ClinGen Allele Registry |
| `Pa` | `PaId`, `"PA..."` | `PA` and a positive number; confirm the form against the registry when built | One protein allele | ClinGen Allele Registry |
| `Vcv` | `VcvId`, `"VCV000013961"` or `"VCV000013961.98"` | `VCV`, nine digits and an optional version without leading zeros | A ClinVar variation record, which may be one allele, a haplotype or a compound | ClinVar accessions |
| `ClinVarVariation` | `ClinVarVariationId`, `"clinvar:13961"` | Positive number with the `clinvar:` prefix | Same as `Vcv`: one allele, a haplotype or a compound | identifiers.org and Bioregistry `clinvar` |
| `Cosmic` | `CosmicId`, `"COSV55386424"`, `"COSM476"`, `"COSN..."` | Kind `COSV`, `COSM` or `COSN` and a positive number; the kind is part of the value | `COSV` is a genomic variant; `COSM` is a legacy mutation on one transcript; `COSN` is a noncoding variant | COSMIC |
| `Vrs` | `VrsId`, `"ga4gh:VA.<32 characters>"` | Prefix and a 32-character base64url digest; kept as given by a source until a crate computes it | One allele as computed from its normalized form | VRS 2.x computed identifier |

`RcvId` (`"RCV000013961.12"`), `ScvId` (`"SCV000061234.3"`) and `ClinVarAlleleId` (a positive number) are not variant identifiers. An RCV names an aggregate of submissions for one variant and condition, an SCV names one submission, and a ClinVar AlleleID names an allele inside a ClinVar record. They appear in `Attribution.record` or in the consumer's ClinVar source record, never in `VariantId`. `RcvId` and `ScvId` share the VCV grammar with their own prefix.

BioMCP's `VariantInput` stays a BioMCP query type. It reads `VariantId` text, `HgvsText` and BioMCP's command conveniences, such as a bare ClinVar number.

A VRS identifier depends on the refget digest of the reference sequence, which needs sequence data. The common crate defines the type. The projecting crate computes it later, after the merge, when the crate can depend on a hash library.

### 3.5 The variant record

`Variant` is the biodata entity for one variant. It holds every fact its identity sources give and no disclosure logic.

```text
Variant {
    ids: [VariantId],                // every identifier the sources state, never inferred
    representations: [Representation], // every source-qualified form
    genes: [GeneRef],                // as the sources state them
}
```

| Type | Fields and wire form | Rule | Standard |
| --- | --- | --- | --- |
| `GeneRef` | `{"hgnc_id"?: HgncId, "ncbi_gene_id"?: NcbiGeneId, "ensembl_gene_id"?: EnsemblGeneId, "label": Text}` | A reference to a gene by the identifiers the source states and its label, usually the symbol as given. No identifier is inferred from the label | HGNC, NCBI Gene, Ensembl |
| `NcbiGeneId` | Positive integer, such as `673` | Replaces `EntrezGeneId` | NCBI Gene |
| `EnsemblGeneId` | `"ENSG00000157764"` or `"ENSG00000157764.14"` | One type with an optional version | Ensembl |

The projecting crate's normalized view extends this with `AlleleKey` and its projections in its own types. A `Statement` may be built as a view from a crate's own reader records, which stay the source for that crate's other writers. BioMCP's card, header and sections pick from `Variant` in BioMCP response types, as architecture section 3.10 says.

### 3.6 Annotations as statements

Every variant, gene or region assertion whose source classification belongs to a proposition this model defines becomes one `Statement`. A statement never merges two sources and never maps one framework onto another. A source classification outside those propositions, such as ClinVar's drug response, risk factor, protective, affects, association, association not found, confers sensitivity, other or not provided, stays in the consumer's native source record with its label, IDs, domain, condition, review status, dates and provenance, and carries `statement_unmapped: true` instead of a `Statement`. The same holds for a source row whose subject the model cannot reference without inventing an identifier, such as an OncoKB implication's alteration group, which carries no source profile record. A new proposition or subject kind for such rows needs both PMs' agreement before either crate builds it.

```text
Statement {
    subject: Subject,               // a variant, a gene or a region
    proposition: Proposition,       // what kind of claim
    method: Method?,                // the guideline the source states it applied
    predicate: Predicate?,          // the claim's direction of effect, when stated
    direction: Direction?,          // supports, disputes, neutral
    classification: Term,           // the source's call, label always kept
    strength: Term?,                // evidence level, tier level or review status
    condition: Concept?,            // disease or tumor type
    therapies: [Concept],           // for therapeutic response
    allele_origin: Term?,           // germline, somatic and the source's other values
    mode_of_inheritance: Term?,     // HPO-coded, as stated
    gene_context: GeneRef?,         // absent when the subject is a gene
    criteria: [Text],               // for example ACMG codes met, as stated
    criteria_not_met: [Text],       // criteria the source evaluated and found not met
    status: Term?,                  // the source's record status label
    citations: [Pmid],
    attribution: Attribution,       // source, record, version, release, submitter, dates
}
```

| Type | Fields and wire form | Rule | Standard |
| --- | --- | --- | --- |
| `VariantRef` | Internally tagged by `"level"`: `allele` with `id` or `hgvs`; `protein` with `gene` and `hgvs` or `label`; `profile` with `record`, `label` and member refs, each an `allele` or `protein` ref; a profile never contains a profile | A reference by ID and label, never an embedded entity. A protein-level claim such as BRAF V600E never becomes a genomic allele. A compound profile stays a profile. A fourth case, `categorical`, names a gene-level category (section 3.10) | Cat-VRS `CategoricalVariant`, its protein-consequence recipe, and CIViC molecular profiles |
| `Subject` | Internally tagged by `"level"`: the four `VariantRef` cases `allele`, `protein`, `profile` and `categorical`, unchanged on the wire; `gene` with the `GeneRef` fields; `region` as `{"level": "region", "interval": GenomicInterval, "label": Text, "record"?: Text}` | The type of `Statement.subject`. One tag holds every case, so a `VariantRef` value is a valid `Subject` with the same JSON. The `region` interval requires its build. A region is never a `CopyNumberChange`, because a dosage region states no copy change. `record` is the source's own ID for the region | Cat-VRS, VA-Spec gene and region subjects |
| `Proposition` | `pathogenicity`, `oncogenicity`, `clinical_significance`, `therapeutic_response`, `diagnostic`, `prognostic`, `functional_impact`, `gene_disease_validity`, `haploinsufficiency`, `triplosensitivity`, `gene_role` | `#[non_exhaustive]`. The last four take a `gene` or `region` subject. A gene-level statement leaves `gene_context` absent. `gene_role` keeps the source label in a `Term` with no coding. `gene_phenotype_association` is rejected, because those rows carry no classification and `classification` is required | VA-Spec proposition classes; ClinGen gene validity and dosage |
| `Predicate` | snake_case forms of the VA-Spec predicates, such as `is_causal_for`, `predicts_sensitivity_to`, `predicts_resistance_to` | Confirm the full list against VA-Spec 1.1 when coded | VA-Spec |
| `Direction` | `supports`, `disputes`, `neutral` | CIViC `DOES_NOT_SUPPORT` maps to `disputes` | VA-Spec `direction` |
| `Term` | `{"label": Text, "coding"?: Coding}` | `label` is the source's whole text, always kept. `coding` is present only when the label is a documented value of a closed vocabulary below | VA-Spec `MappableConcept` |
| `Coding` | Internally tagged by `"system"`, with a checked `"code"` | Each system is one Rust enum of its documented codes | VA-Spec `Coding` |
| `Concept` | `{"id"?: Curie, "label": Text}` | `Curie` is checked `prefix:local` text, such as `MONDO:0005105` or `NCIT:C3224` | VA-Spec `MappableConcept` |
| `Method` | `{"label": Text, "version"?: Text}` | The guideline the source states it applied, such as the ACMG 2015 guideline or a ClinGen SOP version, with its label and version as stated | VA-Spec `specifiedBy` |
| `criteria_not_met` | `[Text]` | Only criteria the source states it evaluated and found not met. A criterion absent from `criteria` is never read as not met | ClinGen criteria assessments |
| `mode_of_inheritance` | `Term` | Coded in system `hpo` only when the label is a listed mode of inheritance term | HPO |
| `status` | `Term` with no coding | The source's record status label as given, such as flagged, rejected or retracted. No boolean replaces it, because those labels mean different things | Source record status |
| `Attribution` | `{"source", "record"?, "version"?, "release"?, "submitter"?, "evaluated"?, "approved"?, "published"?, "url"?}` | `source` is text naming the source family; each consumer keeps its own closed list of sources and writes its spelling here. `record` is the source's own ID for this assertion. `version` is that record's version. `release` is the dataset release as the source labels it. `evaluated`, `approved` and `published` are each a `PartialDate`, only as the source states it | VA-Spec `contributions` and `reportedIn`; the projecting crate's `Provenance`; the standalone crate's `source_release` |

The first coding systems are those with a published closed vocabulary. Each code below is a snake_case spelling. Export to VA-Spec uses the VA-Spec code text, such as `likely pathogenic`.

| `system` | Codes | Source |
| --- | --- | --- |
| `acmg_amp_2015` | `pathogenic`, `likely_pathogenic`, `uncertain_significance`, `likely_benign`, `benign` | Richards 2015; VA-Spec ACMG 2015 profile |
| `clingen_low_penetrance_2024` | `pathogenic_low_penetrance`, `likely_pathogenic_low_penetrance`, `established_risk_allele`, `likely_risk_allele`, `uncertain_risk_allele` | VA-Spec ACMG profile |
| `amp_asco_cap_2017_tier` | `tier_i`, `tier_ii`, `tier_iii`, `tier_iv` | Li 2017; VA-Spec AAC 2017 profile |
| `amp_asco_cap_2017_level` | `level_a`, `level_b`, `level_c`, `level_d` | Same |
| `clingen_cgc_vicc_2022` | `oncogenic`, `likely_oncogenic`, `uncertain_significance`, `likely_benign`, `benign` | Horak 2022; VA-Spec CCV 2022 profile |
| `clinvar_review_status` | ClinVar's review status values | ClinVar XML schema |
| `civic_evidence_level` | `a`, `b`, `c`, `d`, `e` | CIViC |
| `oncokb_level` | `level_1`, `level_2`, `level_3a`, `level_3b`, `level_4`, `level_r1`, `level_r2`, `level_dx1` to `level_dx3`, `level_px1` to `level_px3` | OncoKB API |
| `oncokb_oncogenicity` | `oncogenic`, `likely_oncogenic`, `likely_neutral`, `inconclusive`, `resistance`, `unknown` | OncoKB API |
| `clinvar_classification_type` | `germline`, `somatic_clinical_impact`, `oncogenicity` | ClinVar XML schema |
| `clinvar_allele_origin` | ClinVar's allele origin values in snake_case, such as `germline`, `somatic`, `de_novo`, `maternal`, `paternal`, `unknown` | ClinVar XML schema |
| `so` | Sequence Ontology IDs, such as `SO:0001583` for missense variant, with the SO label | Sequence Ontology; used for consequences and edit kinds |
| `hpo` | Mode of inheritance terms: `HP:0000006` autosomal dominant, `HP:0000007` autosomal recessive, `HP:0001417` X-linked, `HP:0001427` mitochondrial, `HP:0032113` semidominant | Human Phenotype Ontology |
| `clingen_gene_validity_2017` | ClinGen's published classification labels in snake_case, such as `definitive`, `strong`, `moderate`, `limited`, `disputed`, `refuted`, `no_known_disease_relationship`; the list is confirmed against ClinGen's SOP when coded | ClinGen gene-disease validity SOP |
| `clingen_dosage_score` | ClinGen's scores 0, 1, 2, 3, 30 and 40, each as its published label in snake_case | ClinGen dosage sensitivity curation |

Gene-phenotype association rows and ACMG secondary findings list membership stay in the projecting crate's own records and are not statements. ClinVar's record tree, with its submissions, conditions and traits, stays in the consumer as a source record. Biodata holds ClinVar's closed vocabularies above and `Statement` as the shared view of each submission. A ClinVar germline label such as `Likely pathogenic` gets `acmg_amp_2015` coding only when the submission states that method. An aggregate label such as `Conflicting classifications of pathogenicity` keeps its label and has no coding. A CIViC assertion's `Tier I - Level A` becomes `amp_asco_cap_2017_tier` `tier_i` in `classification` and `amp_asco_cap_2017_level` `level_a` in `strength`, both with the CIViC label.

The relation between a query and a statement's subject, such as an exact match, the same codon or the same gene, stays out of `Statement` and travels in the consumer response. The single `level` tag keeps schema changes additive: a statement with a variant subject keeps its JSON under `Subject`, and the new `Statement` and `PopulationFrequency` fields are optional, so `biodata.statement/1` and the population frequency schema keep their major versions.

### 3.7 Population frequency, cohort counts, coverage and predictor scores

| Type | Fields | Standard |
| --- | --- | --- |
| `PopulationFrequency` | One record per stratum, fields below | VA-Spec cohort allele frequency study result. A missing dataset is absent, never zero. No frequency or count is computed from other counts |
| `PredictorScore` | `subject: VariantRef`, `method`, `score` as a finite number, `label`?, `gene`?: `GeneRef`, `transcript`?: `Accession` or `AccessionBase`, `attribution` | Source values as given. No score blending. One record per transcript prediction. `transcript` is an `Accession` when the source gives a version and an `AccessionBase` when it does not; no crate adds or drops a version. A rank score is its own record with its own method, so there is no `rank_score` field. Gene constraint is not a `PredictorScore`; a shared constraint type waits until both crates need one |
| `CohortObservation` | One record per subject and dataset release, fields below | Source counts as given. No denominator or frequency is computed |
| `SiteCoverage` | `{"location": GenomicInterval, "dataset": Text, "dataset_version"?: Text, "data_type"?, "subset"?: Text, "mean"?, "median"?, "allele_number"?}` | The location requires its build. `data_type` and `subset` follow `PopulationFrequency`. Each value appears only as the source states it. Built only by the projecting crate |

A `CohortObservation` holds tumor-cohort counts for one subject, such as GENIE or hotspot counts.

| Field | Type and wire form | Rule |
| --- | --- | --- |
| `subject` | `VariantRef` | The variant or category the counts belong to |
| `dataset` | `Text`, such as `"genie"` | The dataset family |
| `dataset_version` | `Text`, optional | The release only as the source labels it |
| `unit` | `"samples"`, `"patients"`, `"tumors"` | What each count counts |
| `sample_count` | Integer | The source's stated count of units that carry the subject |
| `profiled_sample_count` | Integer, optional | Only a source-stated count of units whose assay covered the locus |
| `cohort_sample_count` | Integer, optional | The cohort size as the source states it. It is never a denominator |
| `by_condition` | `[{"condition": Concept, "count": integer, "profiled_sample_count"?: integer}]` | One entry per condition the source states, in source order |
| `attribution` | `Attribution` | |

Candidate or exact membership and paging stay in the consumer response.

A `PopulationFrequency` holds one stratum of one dataset release. A source that reports exomes, genomes and joint counts, each with ancestry, sex and subset strata, gives one record per combination it states.

| Field | Type and wire form | Rule |
| --- | --- | --- |
| `subject` | `VariantRef` | The allele the counts belong to |
| `build` | `Build` | The build of the dataset's allele |
| `dataset` | `Text`, such as `"gnomad"` | The dataset family |
| `dataset_version` | `Text`, such as `"4.1"` | The release as the source labels it |
| `data_type` | `"exomes"`, `"genomes"`, `"joint"`, optional | The gnomAD sample set; absent for a dataset with one set |
| `ancestry_group` | `Text`, optional, such as `"nfe"` | The dataset's own group code as published; absent means all samples. No code is mapped across datasets |
| `sex` | `"xx"`, `"xy"`, optional | Absent means both |
| `subset` | `Text`, optional, such as `"non_ukb"` | The dataset's own subset code; absent means the full set. No subset code is agreed across datasets |
| `genotype_filter` | `"raw"`, optional | `raw` marks counts before the source's genotype filters; absent means the source's default filtered counts |
| `grpmax_group` | `Text`, optional | Only on the record with no `ancestry_group`. It holds the group the source states as its grpmax, and that group's counts live in its own stratum record. No crate picks the grpmax by comparing groups |
| `allele_count` | Integer, optional | Must not exceed `allele_number` |
| `allele_number` | Integer, optional | |
| `allele_frequency` | Finite number from 0 to 1, optional | Only as the source states it |
| `homozygote_count` | Integer, optional | |
| `heterozygote_count` | Integer, optional | Only as the source states it, never computed from other counts |
| `hemizygote_count` | Integer, optional | Stated for X and Y sites |
| `homoplasmic_count`, `heteroplasmic_count` | Integer, optional | Mitochondrial sites only, as the source states them |
| `homoplasmic_frequency`, `heteroplasmic_frequency` | Finite number from 0 to 1, optional | Mitochondrial sites only, as the source states them |
| `faf95` | Finite number from 0 to 1, optional | The filtering allele frequency at 95 percent confidence, only as the source states it |
| `filters` | `[Text]` | The source's filter flags as given; empty means the source gave no flag |
| `attribution` | `Attribution` | |

### 3.8 Conventions

- Rust names are the type names above. JSON field names are snake_case.
- Enums with data are internally tagged with a field named for the choice: `level`, `system`, `resolution`, `state`.
- Closed sets parse only their canonical spellings. Provider spellings map in each adapter. An unknown provider value leaves the field absent and records a problem, or keeps the source label in a `Term` with no coding.
- `Option` means the source has no value. JSON omits absent fields and empty lists and never prints `null`.
- Every identifier and checked text value serializes as its canonical text and parses back to an equal value.
- Lists hold every stated value in source order. No list is cut in a biodata type.

### 3.9 Who builds what now

| Part | Builder | The other team |
| --- | --- | --- |
| `Accession`, `AccessionBase`, `Base`, `Chromosome`, the build-to-accession table | Projecting crate, already built except `AccessionBase` | BioMCP copies them, renaming nothing |
| `HgvsParsed`, `ProteinChange`, the RNA base type, alteration and cytogenetic notation, normalization, `AlleleKey`, projection, VRS digest computation | Projecting crate | BioMCP uses none of them and passes `HgvsText` to its sources. BioMCP's search query type becomes the private `ProteinQuery` |
| `Transcript`, `TranscriptAlignment`, `TranscriptPosition`, `TranscriptConsequence`, `GeneticCode`, `EditKind`, `Lifted`, `vcf::Site`, `UncertainSpan0`, `UncertainLocation`, `CopyNumberCount`, `CopyNumberChange` | Projecting crate | BioMCP copies each one when a section needs it |
| `HgvsText`, `Expression`, `Representation`, the identifier newtypes, `VariantId`, `Variant`, `GeneRef`, `Mane` | BioMCP | The projecting crate copies them when its readers need them |
| `Statement`, `Term`, `Coding`, `Concept`, `Attribution`, `VariantRef` with its `categorical` case, the ClinVar closed vocabularies | BioMCP, with its ClinVar, ClinGen and CIViC sections | The projecting crate maps its ClinVar and ClinGen readers onto them when it next touches those readers |
| `PopulationFrequency`, `PredictorScore` | BioMCP, with its population and prediction sections | The projecting crate maps its gnomAD and predictor readers |
| `Subject`, the gene propositions and their coding systems, `Method`, the new `Statement` and `Attribution` fields, `CohortObservation`, the new `PredictorScore` and `PopulationFrequency` fields, `UniProtIsoform`, `AminoAcids`, the new `UniProtFeature` fields, `ProteinSite` and the `residue` category | The first crate that needs each one | The other adopts it when it next touches the matching reader, as for `Statement` |
| `SiteCoverage`, `RegulatoryRegion`, gene-phenotype rows, ACMG secondary findings list membership | Projecting crate only | BioMCP does not copy them |
| `Position1`, `Span0`, `Strand`, `Build`, `GenomicInterval`, `SequenceLocation`, `Allele`, the refusal codes in section 3.11 | Both, identical | Each change goes to both PMs before it lands |

BioMCP's landed variant identity work aligns with this plan through its alignment ticket, which follows section 6.

### 3.10 Transcripts, consequences, copy number and categories

These shapes were settled in the 2026-10-24 crate comparison. The four joint additions at the end of this section are agreed by both PMs.

| Type | Fields and wire form | Rule | Standard |
| --- | --- | --- | --- |
| `Mane` | `{"status": "select" \| "plus_clinical", "refseq": Accession, "ensembl": Accession}` | Both accessions versioned. A gene holds a list, `mane: [Mane]`, in both crates. Replaces `ManeSelect` | MANE |
| `Transcript` | `{"accession": Accession, "gene": GeneRef, "protein"?: Accession, "coding"?: Span0, "length": Position1, "mane"?: Mane}` | `coding` is the CDS on the transcript, inside `length` | RefSeq, Ensembl |
| `TranscriptAlignment` | `{"transcript": Accession, "reference": Accession, "strand": Strand, "method": Text, "exons": [AlignedExon]}` | Exons in transcript order | UTA |
| `AlignedExon` | `{"transcript": Span0, "reference": Span0, "cigar": Cigar}` | The CIGAR ops' lengths sum to both spans | UTA |
| `Cigar` | Text such as `"120M2I30M"` | Parsed SAM ops, serialized as text, with the genome as the reference side | SAM |
| `TranscriptPosition` | `{"anchor": Anchor, "offset"?: integer}` | `offset` is the signed intron offset, nonzero, negative before the anchor and positive after it | HGVS |
| `Anchor` | Internally tagged by `"kind"`: `upstream` with `distance: Position1` for `c.-N`; `coding` with `position: Position1` for `c.N`; `downstream` with `distance: Position1` for `c.*N`; `noncoding` with `position: Position1` for `n.N` | `c.0` and `n.0` cannot be written. This is the only Anchor type in either crate; the HGVS grammar uses it too. An unknown position such as `c.?` is an unknown position in the grammar, not an Anchor case | HGVS |
| `Region` | `{"kind": "exon" \| "intron", "number": Position1, "total": Position1}` | `number` is at most `total` | VEP and HGVS exon numbering |
| `EditKind` | A `Coding` in system `so`, such as `SO:1000002` substitution | Only exact SO variant-class terms | Sequence Ontology |
| `NaEdit` | The projecting crate's parsed nucleotide edit | Built only by its grammar | HGVS |
| `GeneticCode` | NCBI translation table number: `1` standard, `2` vertebrate mitochondrial | A fixed codon table per number with no I/O | NCBI translation tables |
| `Lifted` | Internally tagged by `"state"`: `lifted` with `build: Build`, `allele: Allele` and `chain: Text`; `unlifted` with `reason: Text` and `chain: Text` | `chain` names the chain file release. `reason` is the liftover tool's stated reason | UCSC chain files |
| `vcf::Site` | `{"chrom": Contig, "pos": Position1, "id": [Text], "ref": Sequence, "alt": [Alt], "qual"?: number, "filter": [Text]}` | `ref` is padded and nonempty. `Alt` is internally tagged by `"kind"`: `sequence`, `symbolic` with its text, `breakend` with its text, `missing` for `.`, `overlap` for `*`. No raw line is kept | VCF 4.3 |
| `UniProtIsoform` | Canonical text such as `"P15056"` or `"P15056-2"` | A UniProt accession with an optional isoform number, kept as stated. No crate adds `-1` or invents a version | UniProt |
| `AminoAcids` | Text of one-letter amino-acid codes, such as `"V"` or `"E"` | Uppercase and nonempty. The 20 standard codes plus `U`, `O` and `X`. It is not the DNA `Sequence` type | IUPAC amino-acid codes |
| `UniProtFeature` | `{"protein": UniProtIsoform, "span": UncertainSpan0, "kind": Term, "description"?: Text, "id"?: Text, "original"?: AminoAcids, "alternative": [AminoAcids], "evidence": [Text]}` | `kind` keeps the UniProt label and codes it only when the label is a documented feature type. `original` and `alternative` hold a variant or conflict feature's residues as stated | UniProt |
| `ProteinSite` | `{"gene": GeneRef, "transcript"?: Accession \| AccessionBase, "protein"?: Accession, "position": Position1, "end"?: Position1, "residue"?: Text}` | A residue or residue range, not a variant. `end` is at least `position`. No crate adds a protein accession the source does not give | Cancer hotspots |
| `RegulatoryRegion` | The projecting crate's region record, with a `kind: Term` | `kind` is coded in system `so` only for an exact SO term. Built only by the projecting crate | Sequence Ontology |
| Consequence | `consequences: [Coding]` in system `so` | A set of SO codings with no duplicates. A source term with no exact SO term keeps its label in a `Term` with no coding; no nearest term is guessed. A consumer's ordering of mutation types stays in the consumer. A data failure in the source is a refusal, not a consequence | Sequence Ontology |
| `TranscriptConsequence` | `{"transcript": Accession, "gene"?: GeneRef, "consequences": [Coding], "coding_hgvs"?: HgvsText, "protein_hgvs"?: HgvsText, "region"?: Region, "attribution": Attribution}` | Joint addition. One record per transcript a source annotates. `consequences` is nonempty | VEP and SnpEff output, VA-Spec molecular consequence |
| `Bound` | `{"min"?: integer, "max"?: integer}` | Joint addition. At least one side present; `min` at most `max`; equal sides mean an exact bound | VRS `Range` |
| `UncertainSpan0` | `{"start": Bound, "end": Bound}` | Joint addition. Zero-based interbase, like `Span0` | VRS `SequenceLocation` with ranges |
| `UncertainLocation` | `{"sequence": Accession, "span": UncertainSpan0}` | Joint addition. Also the location of a span read from ISCN text, on the `NC_` accession for the stated build | VRS `SequenceLocation` |
| `CopyNumberCount` | `{"location": UncertainLocation, "copies": Bound}` | Joint addition | VRS 2.x `CopyNumberCount` |
| `CopyNumberChange` | `{"location": UncertainLocation, "copy_change": CopyChange}` | Joint addition. `CopyChange` cases: `complete_genomic_loss`, `high_level_loss`, `low_level_loss`, `loss`, `regional_base_ploidy`, `gain`, `low_level_gain`, `high_level_gain`, each carrying its EFO code, confirmed against VRS 2.x when coded | VRS 2.x `CopyNumberChange` |
| `VariantRef` `categorical` | `{"level": "categorical", "gene": GeneRef, "category": Category, "partner"?: GeneRef, "region"?: Region, "edit"?: Coding, "label": Text}` | Joint addition. `Category` cases: `amplification`, `deletion`, `fusion`, `exon_mutation`, `oncogenic_mutation`, `residue`. `partner` appears only on `fusion` and may be absent when the source names none. `region` and `edit` appear only on `exon_mutation`, with `edit` in system `so`. `residue` carries `site: ProteinSite` and names every variant at that site. `label` keeps the source text, such as `EGFR exon 19 deletion` | Cat-VRS categorical variants and their recipes; `residue` follows the location-defined recipe |

### 3.11 Refusal codes

Every shared refusal uses one of these codes. Codes have the form `area.thing.problem`. Messages never hold input text. A crate adds a code to this list through both PMs before it lands.

| Area | Codes |
| --- | --- |
| Coordinates | `coord.position.zero`, `coord.position.overflow`, `coord.span.order`, `coord.span.overflow`, `coord.bound.empty`, `coord.bound.order`, `coord.build.unknown`, `coord.chromosome.invalid`, `coord.contig.invalid`, `coord.strand.unknown`, `coord.interval.build_absent`, `coord.interval.build_mismatch`, `coord.interval.chromosome_mismatch` |
| Accessions | `accession.prefix.unknown`, `accession.digits.width`, `accession.version.missing`, `accession.version.leading_zero`, `accession.version.overflow`, `accession.version.present`, `accession.lrg.invalid` |
| Bases and sequences | `sequence.base.invalid`, `sequence.base.rna` |
| Alleles | `allele.state.empty`, `allele.reference.length` |
| HGVS text and expressions | `hgvs.text.empty`, `hgvs.reference.missing`, `hgvs.reference.invalid`, `hgvs.inner_reference.invalid`, `hgvs.syntax.unknown`, `hgvs.body.empty`, `hgvs.body.unprintable`, `expression.syntax.unknown`, `expression.value.invalid` |
| Variant identifiers | `id.rs.invalid`, `id.ca.invalid`, `id.pa.invalid`, `id.vcv.invalid`, `id.rcv.invalid`, `id.scv.invalid`, `id.clinvar_variation.invalid`, `id.clinvar_allele.invalid`, `id.cosmic.invalid`, `id.vrs.invalid`, `id.variant.unknown` |
| Gene identifiers | `id.hgnc.invalid`, `id.ncbi_gene.invalid`, `id.ensembl_gene.invalid`, `id.omim.invalid`, `gene.symbol.invalid`, `gene.ref.label_empty`, `gene.mane.status_unknown` |
| Transcripts and alignments | `transcript.coding.outside`, `transcript.anchor.unknown`, `transcript.offset.zero`, `transcript.region.number`, `alignment.cigar.invalid`, `alignment.cigar.empty`, `alignment.cigar.length`, `alignment.exon.order`, `alignment.exon.overlap` |
| Fixed standards | `genetic_code.table.unknown`, `genetic_code.codon.invalid`, `consequence.so.unknown`, `consequence.list.duplicate` |
| Population and scores | `population.count.exceeds_number`, `population.frequency.range`, `population.mt_field.misplaced`, `population.grpmax.misplaced`, `number.value.non_finite` |
| Copy number and categories | `copy_number.change.unknown`, `variant_ref.category.unknown`, `variant_ref.partner.misplaced`, `variant_ref.region.misplaced` |
| Proteins | `protein.residue.invalid`, `protein.site.order`, `id.uniprot_isoform.invalid` |
| VCF | `vcf.ref.empty`, `vcf.alt.invalid`, `vcf.chrom.invalid` |

`accession.version.present` refuses a version in `AccessionBase` text. `coord.interval.build_absent` refuses containment or overlap when either interval has no build, and refuses a `region` subject or a `SiteCoverage` location with no build. `sequence.base.rna` refuses `U` in DNA text. `population.mt_field.misplaced` refuses a homoplasmic or heteroplasmic field on a subject outside the mitochondrial chromosome. `population.grpmax.misplaced` refuses a `grpmax_group` on a row that also names an `ancestry_group`, because the group with the maximum frequency describes a whole-cohort row. `protein.residue.invalid` refuses `AminoAcids` text outside its codes. `id.uniprot_isoform.invalid` refuses malformed `UniProtIsoform` text. `protein.site.order` refuses a `ProteinSite` whose `end` is before its `position`. BioMCP's current two-part codes, such as `position.zero` and `build.unknown`, move to these codes under its alignment ticket.

## 4. Rules both teams follow now

1. **Names.** Both crates use the type names in section 3. A type with one of these names has the meaning given here. A new public variant or annotation type gets a name both PMs accept before it lands.
2. **Wire spellings.** JSON fields are snake_case. Enum values are snake_case, except published spellings that are themselves the standard: `GRCh37`, `GRCh38`, `+`, `-`, chromosome names, VRS expression syntaxes and identifier text.
3. **Coordinate labels.** A field holding a coordinate has a type that names its counting rule: `Position1` or `Span0`. No bare integer holds a coordinate. Interbase is the stored form for alleles. A protein residue position is a `Position1`, and a protein location is a `SequenceLocation` on a protein accession.
4. **Identifier formats.** Identifiers use the wire text in section 3.4 and the accession rules in section 3.1. Accessions in variant values carry their version. An rsID is never an allele join key.
5. **Schemas.** Both crates name schemas `biodata.<entity>/<major>`, such as `biodata.allele/1` and `biodata.statement/1`. The projecting crate adds generated schemas when it first serializes a common type in output. Removing, renaming or retyping a field raises the major version.
6. **Serde.** Deserialization runs the same checks as construction. No `flatten`. Unknown fields are refused.
7. **No disclosure logic in biodata.** Headers, cards, sections, caps, pages and next steps are consumer response types, outside the shared model.
8. **No interpretation in biodata.** No type ranks, votes on or merges classifications. No constructor fetches, normalizes or checks a reference base.
9. **Error codes.** Codes are dotted, `area.thing.problem`, and shared refusals use the list in section 3.11, such as `allele.reference.length` or `id.rs.invalid`. Messages never hold input text.
10. **Shared public fixtures.** Both crates carry the same small fixture file of public variants under `tests/fixtures/variant/`, with identical bytes. Each row gives the input text and its expected values in each form: `Allele` JSON, SPDI, VCF, gnomAD ID, HGVS text and identifiers. Each expected value is checked against a public source when the row is added. Both crates run every row through their copies. The first rows are:
    - BRAF c.1799T>A: `NC_000007.13:g.140453136A>T`, `NC_000007.14:g.140753336A>T`, `NM_004333.4:c.1799T>A`, `NP_004324.2:p.Val600Glu`, `CA123643`, `clinvar:13961`.
    - rs113488022 with its three alternate alleles.
    - EGFR L858R as MyVariant's `chr7:g.55259515T>G` on GRCh37.
    - An EGFR exon 19 deletion: `NC_000007.13:g.55242469_55242471del`.
    - An insertion, with its empty-reference `Allele` and padded VCF form.
    - TP53 `NM_001126114.2:c.1015G>C`, which resolves.
    - TP53 `NC_000017.10:g.7576633A>G`, which the registry rejects for a wrong reference base.
    - CHEK2 `NM_007194.4:c.1100del`.
    - TERT `NM_198253.3:c.-124C>T`.
    - One mitochondrial variant on `NC_012920.1`, whose `AlleleKey` rows for GRCh37 and GRCh38 differ only by build.
    - One reference allele whose reference equals its alternate, which every crate accepts.
    - One allele text containing `U`, which every crate refuses.
    - One ClinVar germline statement, one CIViC assertion with a tier and level, and one ClinGen expert panel interpretation, each with source labels.
11. **Copies.** A copied file keeps the owner's spelling and tests. A fix goes to the owner first, through the PMs, and the copy follows.
12. **Public text.** Shared types and docs name no consumer project.
13. **Legacy writers.** A writer that must reproduce an existing product's output byte for byte, including its nulls, empty rows and chromosome prefixes, is outside rules 2, 5 and 8 of this list and outside section 3.8. The shared types feed it; its output keeps the legacy shape.
14. **Lenient chromosome input.** A crate may accept provider spellings such as `chr7`, `M` or `23` in one adapter helper that returns a `Chromosome`. The `Chromosome` type itself parses only its canonical spellings.
15. **Biodata holds identifiers and closed vocabularies.** Identifiers, coordinates, transcripts, exons, alignments, edit kinds, MANE status, liftover results, VCF sites, consequence codings, gene references, population frequency, cohort counts, protein features, protein sites, protein changes, closed vocabularies such as ClinVar's review status, classification type and allele origin, and fixed standards such as the genetic code live in biodata. A source's record tree, such as a ClinVar record with its submissions and traits, stays in the consumer that reads it. `Statement` is the shared shape for what such a record asserts. Biodata does no I/O and no data lookup.
16. **Neutral shapes.** No biodata type is shaped by an index file format, a database or ORM layout, or one API's response. Each adapter, index codec or product writer translates between its format and the neutral type, and product-specific spellings live in that application as tables over neutral values.
17. **One name per concept.** When both crates have a concept, they share one name and the union of fields, and any difference in meaning is settled in this plan before code lands, never by a second type. The crate that introduces a concept first proposes its name and shape, and either PM may reject any proposal, including one already agreed, when a better design serves both projects; the reasons are written in this plan, and a disagreement the PMs cannot settle goes to Ian through Jarvis. Current names: BioMCP's `HgncId`, `NcbiGeneId`, `OmimId`, `EnsemblGeneId`, `GeneRef`, `Mane`, `PopulationFrequency`, `VariantId` and `Statement`; the projecting crate's `Accession`, `AccessionBase`, ISCN and alteration grammar, `Transcript`, `TranscriptAlignment`, `TranscriptPosition`, `Cigar`, `GeneticCode`, `ProteinChange`; the source-gap names `Subject`, `Method`, `CohortObservation`, `UniProtIsoform`, `AminoAcids` and `ProteinSite`; the projecting crate's own `SiteCoverage` and `RegulatoryRegion`; and the identifier newtypes `RsId`, `CaId`, `PaId`, `VcvId`, `RcvId`, `ScvId`, `ClinVarVariationId`, `ClinVarAlleleId`, `CosmicId` and `VrsId`. `VariantId` is the enum over the variant-level newtypes in section 3.4, each case wrapping its newtype.
18. **Entity versus reference.** A full entity record and a reference to it are different types: `Gene` is the full gene record and `GeneRef` carries only identifiers and a label, as `VariantRef` does for variants. A gene extent is a `GeneRef` beside a `GenomicInterval`. Consequences and edit kinds are sets of Sequence Ontology codings, used only where an exact SO term exists.

## 5. Merge and wire style

Ian ruled on 2026-10-24:

- **Merge.** Ian decides when and where the crates merge, after both projects are done. Neither team plans or proposes the merge. The likely home is a new public crate without commit history. Until then each team keeps its own crate and follows this plan.
- **Wire style.** JSON field names stay snake_case as in section 3. A GA4GH VRS and VA-Spec export, which uses camelCase, can be added later as a fixed field map.

## 6. Settled in the 2026-10-24 crate comparison

Each line is one settled point and its reason. Sections 3 and 4 carry the full shape.

- BioMCP's search type `ProteinChange` becomes the private `ProteinQuery`, because the shared name belongs to the parsed HGVS protein change.
- `Strand` Rust cases are `Plus` and `Minus` with wire `+` and `-`, because both crates need one Rust name and the wire already follows GFF3.
- `Build` Rust cases are `GRCh37` and `GRCh38` and the enum is `#[non_exhaustive]`, because the case names should match the published names and a new build must not break callers.
- `GeneRef { hgnc_id?, ncbi_gene_id?, ensembl_gene_id?, label }` replaces bare symbols in `Variant.genes` and `Statement.gene_context`, because a symbol alone is ambiguous across releases.
- A gene extent is a `GeneRef` beside a `GenomicInterval`, because a separate extent type would repeat both.
- `Accession` is the only versioned accession grammar, with exactly six or nine RefSeq digits and a required version, because two grammars would accept different text for one concept.
- `AccessionBase` is the unversioned twin, and `RefSeqTranscript` and `EnsemblTranscript` fold into the pair, because some sources give no version and that fact must stay visible in the type.
- Accession versions with leading zeros, such as `.04`, are refused, because each accession needs one canonical text.
- Refusal codes take the form `area.thing.problem` from the list in section 3.11, because both crates must refuse the same input with the same code.
- `GenomicInterval.build` is `Option<Build>`, and containment refuses when either build is absent, because a source may state no build and a guessed build misplaces coordinates.
- `HgvsText` accepts an inner accession or gene in parentheses and still refuses text with no reference, because published HGVS allows that form.
- `PopulationFrequency` is one record per stratum with the fields in section 3.7, because a dataset reports counts per ancestry group, sex, subset and sample set, and no frequency may be computed from counts.
- `VcvId`, `RcvId`, `ScvId`, `ClinVarVariationId`, `ClinVarAlleleId`, `PaId` and `CosmicId` with its COSV, COSM or COSN kind are separate newtypes, because each has its own grammar and meaning.
- `VariantId` holds only variant-level identifiers (`Rs`, `Ca`, `Pa`, `Vcv`, `ClinVarVariation`, `Cosmic`, `Vrs`), and each case documents its level, because an rsID names a site, a CA names one allele and a ClinVar variation may be a haplotype or compound.
- `RcvId`, `ScvId` and `ClinVarAlleleId` go to attribution or the source record, because they name aggregates, submissions or record parts and not variants.
- An rsID is never an allele join key, because one rsID can carry several alternate alleles.
- `Base` is DNA only and an RNA base type lives in the projecting crate's grammar, because only that grammar reads RNA notation.
- `Mane { status: select | plus_clinical, refseq, ensembl }` is held as a list per gene in both crates, because a gene can have a MANE Select and a MANE Plus Clinical transcript.
- `GeneSymbol` takes the stricter grammar only if every approved HGNC symbol passes, because a grammar that refuses a real symbol is wrong.
- Consequence is a set of SO codings in system `so`, with no nearest-term guess, because only an exact SO term keeps the source's meaning; mutation-type ordering stays in the consumer and a source data failure is a refusal.
- A placement type beside `Allele` is rejected, and a pure `Allele::trimmed()` is allowed, because one allele shape is enough and trimming needs no reference.
- `Lifted` holds the target `Build` and `Allele` or an unlifted reason, with the chain release, because a liftover result must say which chain made it.
- Hotspot positions and residue ranges use `Position1` in `ProteinSite`, because protein coordinates need the same counting rule as nucleotide ones.
- ClinVar's record tree stays in the consumer, and its review status, classification type and allele origin are closed vocabularies in biodata, because the tree is one source's layout and the vocabularies are reusable.
- `GeneticCode` is keyed by NCBI translation table and holds tables 1 and 2, because human nuclear and mitochondrial translation need only those two.
- `TranscriptPosition` is an HGVS anchor (`c.-N`, `c.N`, `c.*N`, `n.N`) plus a signed intron offset, because that is the HGVS position model and it covers positions after the stop codon.
- `Region` is `{kind, number, total}`, because exon and intron numbers mean little without the total.
- Inheritance is a `Term` coded with HPO mode of inheritance terms, because HPO publishes the closed set.
- `NcbiGeneId` replaces `EntrezGeneId`, because NCBI now names the database Gene.
- `EnsemblGeneId` is one type with an optional version, because sources give both forms of one identifier.
- `Transcript` and `TranscriptAlignment` take the shapes in section 3.10, with CIGAR held as parsed SAM ops serialized as text and the genome as reference, because the alignment must round-trip without a database layout.
- `EditKind` carries SO variant-class terms and `NaEdit` is the parsed edit, because the class and the edit are different facts.
- `vcf::Site` holds `Contig`, a padded `Sequence`, a typed ALT enum and the ID column, and drops the raw line, because the site must be a neutral value and not a file row.
- A UniProt feature holds its isoform, an uncertain span, a `Term` kind, description, ID and evidence, because UniProt feature bounds can be uncertain.
- `TranscriptConsequence`, uncertain bounds (`Bound`, `UncertainSpan0`, `UncertainLocation`), `CopyNumberCount` and `CopyNumberChange`, and the `categorical` `VariantRef` case are joint additions agreed by both PMs, because both crates read sources that state these facts and VRS 2.x and Cat-VRS give their shapes.
- `Statement.subject` is a `Subject` with one `level` tag over the four `VariantRef` cases, `gene` and `region`, and the gene propositions `gene_disease_validity`, `haploinsufficiency`, `triplosensitivity` and `gene_role` join with codings `clingen_gene_validity_2017` and `clingen_dosage_score`, because ClinGen validity and dosage curate genes and regions; a region is not a `CopyNumberChange` because a dosage region states no copy change, and `gene_phenotype_association` is rejected because those rows carry no classification.
- `PredictorScore` keeps a `VariantRef` subject and gains `gene` and `transcript`, one record per transcript prediction, with no `rank_score`, because a rank score is its own record with its own method and gene constraint waits until both crates need a shared type.
- `CohortObservation` holds stated sample, profiled and cohort counts with a unit and per-condition counts, because tumor-cohort sources state counts and no crate may compute a denominator.
- `SiteCoverage` is built only by the projecting crate, because only that crate reads coverage and all-sites allele numbers.
- `PopulationFrequency` gains a stated `heterozygote_count`, mitochondrial homoplasmic and heteroplasmic fields, a stated `grpmax_group` and `genotype_filter`, while `subset` keeps the dataset's own code, because counts must appear only as stated and subset codes differ across datasets.
- `Statement` gains `method`, `criteria_not_met`, `mode_of_inheritance` and an uncoded `status`, and `Attribution` gains `approved` and `published`, because flagged, rejected and retracted mean different things and a criterion absent from `criteria` is not a criterion found not met.
- `UniProtIsoform` keeps stated isoform text, `UniProtFeature` gains `original` and `alternative` as `AminoAcids`, `ProteinSite` becomes a gene-qualified residue with an optional end, `Category` gains `residue`, and a regulatory region kind is an SO-coded `Term` on a `RegulatoryRegion` built only by the projecting crate, because hotspots name residues and not variants and no crate may add an accession or version the source does not give.
