# No text heuristics

Date: 2026-10-24

Decided by Ian, after an audit of BioMCP 0.9 found about 45 text hacks.

BioMCP builds a request, sends it, reads the reply, fills the biodata types and prints the envelope. A view may render that envelope as Markdown. Nothing else sits between the agent's input and the source.

BioMCP never guesses meaning from text. It has no regexes or word shapes that spot genes, drugs, diseases or variants in free text, no hand-made term, alias, synonym or stopword lists, no query rewriting or retries with altered queries, no intent detection or keyword routing, no score blends or rerankers, and no special cases for particular inputs.

These are allowed because they are mechanical or come from a source:

- A typed parse of an input the agent marked by its option, such as `--variant P245S` or an `HGNC:` prefix, using a published standard such as the amino-acid codes or HGVS.
- A source's own lists and matching rules, such as HGNC symbols, previous symbols and aliases, returned as candidates when they tie.
- Fixed, echoed request construction, such as the order of terms in a PubMed query.
- Identifier equality for removing duplicates, and stated layout rules such as round-robin by source rank.
- A sort the agent asks for, such as `--sort citations`.

What 0.9 used heuristics for, 1.0 gets from sources: LitVar2 and Europe PMC full text for variant spellings, MONDO, MeSH or UMLS synonyms for diseases, and the drug sources for drug names. Ambiguity comes back as candidates, never as a guess.

## Amendment, 2026-11-06

A drug name search may run the other names that drug sources return for that exact name, and the reply lists the names searched (0717). The names come from the sources, not from a hand-made list, and nothing is guessed from free text. This narrows "no query rewriting" for that case only. Ian asked for it after the life sciences test.
