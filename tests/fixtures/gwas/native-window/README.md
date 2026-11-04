# Retained GWAS association pages

`page0.body` and `page1.body` are unmodified native response bytes. `provenance.json` records their original requests, dates and hashes. The responses identify six associations for publication 17529967 across two native pages.

The tools-only source value proof uses these bodies through a local TLS request stub. The request lines are synthetic transport declarations matching the captured query selectors. The stub refuses every other request and makes no provider calls.

The proof derives corruptions in memory. The metadata case replaces one `first_author` value with an object. Identity cases replace every first-page `association_id` with an object or zero. Scope cases also replace one requested `pubmed_id` with a contradictory string or object, or change the self-link publication selector. These are synthetic failure witnesses derived from the captures, not provider responses.

The proof checks native positions and continuation, metadata omission, selector refusal, and shared-cache offline behavior. A page with refused identities must not enter the cache, while the unmodified following page remains readable offline.
