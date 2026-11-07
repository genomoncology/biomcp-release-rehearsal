# Look up a stated symptom term

## Question

Which source term represents the stated word Headache?

## 1. Search the stated term

```sh
biomcp search phenotype headache --source monarch
```

Read all returned candidate identifiers and labels. Inspect the echoed query, source outcomes and problems. Select the general term Headache because that is the term in this question. Do not choose a more specific candidate without that information in the question.

## 2. Inspect the selected identifier

```sh
biomcp get phenotype HP:0002315
```

Compare the selected identifier, name, definition and source. The [HPO release 2026-09-01](https://raw.githubusercontent.com/obophenotype/human-phenotype-ontology/master/hp.obo) identifies HP:0002315 as Headache and supplies its definition, citing PMID:15304572. Keep the source identifier with the term.

## Next

If the definition does not match the stated term, return to the candidates and inspect the appropriate source records. Follow returned page commands when candidates remain. This lookup supplies terminology; it establishes no diagnosis, cause, severity or patient interpretation.

Read response status, problems and source terms at every step. Live replies can change.
