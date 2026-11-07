# Follow an article through its sources

## Question

Which source records and citation context can I read after opening an article?

## 1. Retrieve and inspect

```sh
biomcp get article 33123537
```

Read the PMID, DOI, title, publication date and publication type. Inspect abstract parts with their labels and source outcomes.

## 2. Retrieve and inspect

```sh
biomcp article references 33123537 --source semantic-scholar
```

Read the source attribution, referenced publication identities and pagination. A reference edge records a citation relationship.

## 3. Retrieve and inspect

```sh
biomcp article citation-evidence 33123537 17611497
```

Read the returned citation context and its source. Check that both publication identities match the requested direction.

## Next

Open relevant cited articles through returned commands. Read the study’s own methods and results before using a claim; citation direction alone does not express support.

Read response status, problems and source terms at every step. Live replies can change.
