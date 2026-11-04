# BioMCP types

This crate owns BioMCP queries, source identities, response disclosure and their generated schemas. Neutral entity values remain in biodata.

Schema definitions include generic type parameters in their names. Reply and value schemas prefix every definition with `Reply_` and update local references. Query schemas keep unprefixed definitions. Each definition name has one complete body across the generated schemas. Contract IDs keep their published spelling.

`Source`, `Reason` and `ProblemCode` use canonical names for parsing, display and JSON. `Text` trims surrounding whitespace and rejects empty text. JSON deserialization uses the same checks as construction. `Error` returns a stable code and retains no rejected input.

`Problem` describes a field that could not be accepted from a source record. Its optional `record` uses checked text and is omitted from JSON when absent. Unknown JSON fields are rejected.

```rust
use biomcp_types::{Problem, ProblemCode, Source};
use biodata::Text;

let problem = Problem {
    source: "pubmed".parse::<Source>()?,
    record: Some(Text::new(" record-123 ")?),
    field: "publication_type".into(),
    code: ProblemCode::ValueUnknown,
};
assert_eq!(problem.record.as_ref().map(Text::as_str), Some("record-123"));
# Ok::<(), biodata::Error>(())
```
