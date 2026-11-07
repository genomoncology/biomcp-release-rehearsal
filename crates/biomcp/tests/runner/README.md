# Case assertions

The case target discovers YAML files as named tests. `--cases DIR` selects an
external case directory. Cargo filters and `--list` retain those relative names.
Each case shares one isolated environment across its steps.
An explicit name filter that selects no tests exits nonzero and names the filter on stderr.

`scripts.trials.run_selected` selects exact names from an existing executable's
`--list` output before its build preparation callback runs. Pass the target's
existing mode arguments and required working directory. The helper keeps that
command and directory for each exact run. Group names and names owned by another
target fail before preparation. `--cases` selects a directory.

`expect` maps JSON pointers to assertions. Scalars and arrays mean equality.
Objects contain operators combined with AND; use `equals` for a literal object.
The runner validates every step and assertion before launching any command.
Unknown operators, invalid shapes and malformed pointers fail the selected case.

Implemented operators:

| Operator | Expected value | Meaning |
| --- | --- | --- |
| `equals` | Any JSON value | Exact equality, including objects |
| `omits` | Nonempty array of field names | The selected object lacks every named field, including null values |
| `includes` | Any JSON value | An array contains an equal member |
| `count` | Nonnegative integer | Exact array length |
| `rank_of` | Nonempty object of string members to positive integers | First one-based position of each exact string member |
| `max_bytes` | Nonnegative integer | Ceiling on compact JSON bytes |
| `max_tokens` | Nonnegative integer | Ceiling on compact JSON bytes divided by four, rounded up |

Each step also accepts `max_bytes` for raw stdout and `max_tokens` for compact
JSON. By default, registered source policy fields are excluded from these step ceilings.
Set `full_output_bytes: true` to count the complete emitted CLI stdout, including
its newline, or the actual MCP content text. Its token estimate uses those same
bytes divided by four, rounded up. RPC framing remains separate.
An explicit CLI `--format md` or `--format=md` before a literal `--` accepts
root `expect` assertions with `includes` for one substring or `includes_all_of`
for an array of required substrings. Markdown `max_bytes` preserves the comparable text metric by validating and
subtracting publisher policy columns with the existing source terms checker.
It requires an immediately preceding actual JSON CLI step with matching arguments
apart from explicit format options. A missing or different pair fails. With
`full_output_bytes: true`, the ceiling counts every raw byte and requires no pair.
JSON pointers, other operators, MCP parity and token ceilings remain unavailable
for that text assertion form. Literal `stdout` still compares the complete output.

Arrays are ordered. Membership ignores multiplicity. Missing paths fail;
explicit null is a value. Ordinary empty arrays can equal `[]`, have count zero,
and fail membership normally. A missing rank member fails.

A `*` pointer segment expands an array in order. Further segments must exist with
the correct container on every selected branch. Multiple expansions flatten to
one result array. Zero-result wildcard selections fail before assertions run.
Wildcards apply only to arrays. Other segments follow JSON Pointer escaping.
The runner rejects wildcard pointers paired with literal `[]`, `equals: []` or `count: 0`
when it prepares assertions. Assert on the parent array instead, such as
`expect: {'/items': {equals: [], count: 0}}`.

The schema cases demonstrate the implemented syntax against real command output.

## Approved syntax for future cases

The following operators and ranges remain unimplemented until a harvested case
requires them. An assertion using one currently fails as an unknown operator or
invalid exact value.

| Operator | Approved meaning |
| --- | --- |
| `subset` | Recursively match the expected object's keys; compare non-object leaves with exact equality |
| `includes_all_of` | Load a JSON array from the named floor file and require every value as an equal array member; ignore multiplicity |
| `before: [A, B]` | Require distinct present values with the first position of A before the first position of B |
| `unique: true` | Compare whole array member values for uniqueness |
| `regex` | Search with the supplied pattern; authors supply anchors when needed |

Future `count` and `rank_of` bounds accept an exact unsigned integer, an anchored
comparison string using `<`, `<=`, `>` or `>=`, or an inclusive `{min, max}` object
with at least one bound. Unknown bounds and reversed bounds fail validation.
Ranks remain one-based and absent members still fail.

A floor path must be nonempty, relative to the case's parent and nonabsolute.
Normal parent traversal supports external case directories. The loaded JSON must
be an array. An empty floor is allowed but cannot override wildcard-empty
failure. Future code constants must bound floor reads, regex source size and
compiled regex size. Every operator, pattern and floor must validate before any
command launches.

## Diagnostic observations

The optional `--observe PATH` creates an exclusive bounded JSON-lines diagnostic file. It contains existing transport identities, credential-free prepared request URLs, attempts, status, received bytes, elapsed time and sampled raw field structure. It contains no HTTP headers or response body values.

The release artifact gate sets `BIOMCP_TEST_EXECUTABLE` to an absolute installed binary. The shared launcher uses it for CLI, MCP and process checks, including nested runner checks. `BIOMCP_TEST_FIXTURES` selects the complete copied public fixture tree outside the checkout. The runner registers all feature trials even when `--cases` selects another directory. Cache publication, HTTP ownership and AlphaGenome local TLS checks run as their ordinary feature trials.

The scheduled live check runs the built command directly and is described in `docs/development.md`. It does not use this runner.
