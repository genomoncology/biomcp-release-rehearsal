pub(super) fn cases() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "markdown-wrong-pair",
            "  - run: [help, --format, md]\n    max_bytes: 710",
            "comparable Markdown max_bytes requires an adjacent JSON CLI step with matching arguments",
        ),
        (
            "markdown-content",
            "  - run: [help, author, --format, md]\n    expect: {'': {includes: nonexistent-review-text}}",
            "stdout includes: expected text",
        ),
        (
            "markdown-ceiling",
            concat!("  - run: [help, author, --format, json]\n",
                    "  - run: [help, author, --format=md]\n    max_bytes: 0"),
            "stdout max_bytes: expected <= 0",
        ),
        ("markdown-literal-separator", "  - run: [help, --, --format, md]\n    stdout: wrong", "literal stdout requires a plain-text local command"),
        ("literal-stdout", "  - run: [version]\n    stdout: wrong version", "stdout: expected"),
        ("mixed-stdout-json", "    stdout: wrong\n    expect: {'/type': string}", "literal stdout requires one explicit CLI call"),
        ("query-literal-stdout", "    stdout: '{}'", "literal stdout requires a plain-text local command"),
        ("stdin-follow", "    stdin: synthetic\n    follow: page", "literal stdin requires an explicit CLI call"),
        ("stdin-walk", "    stdin: synthetic\n    walk: {items: /items, max_pages: 1}", "literal stdin requires an explicit CLI call"),
        ("excluded-member", "    expect: {'/x-extensible-enum': {excludes: hgnc}}", "excludes: unexpected member"),
        ("excluded-wrong-type", "    expect: {'/type': {excludes: hgnc}}", "excludes: expected an array"),
        (
            "mcp-wrong-value",
            "  - mcp: [schema, biomcp.source/1]\n    expect: {'/type': integer}",
            "/type: expected",
        ),
        ("mcp-bytes", "  - mcp: [schema, biomcp.source/1]\n    max_bytes: 0", "max_bytes: expected <= 0"),
        (
            "mcp-tokens",
            "  - mcp: [schema, biomcp.source/1]\n    max_tokens: 0",
            "max_tokens: expected <= 0",
        ),
        (
            "mcp-pointer",
            "  - mcp: [schema, biomcp.source/1]\n    expect: {'/x-extensible-enum': {count: 0}}",
            "count: expected 0",
        ),
        ("mcp-is-error", "  - mcp: [schema, missing/1]", "MCP is_error mismatch"),
        ("unequal-success", "    mcp: [schema, biomcp.error/4]", "CLI/MCP parity: unequal results"),
        ("no-command", "  - expect: {}", "step requires run, mcp or mcp_call"),
        ("unknown", "    expect: {'/type': {equal: string}}", "unknown assertion operator equal"),
        ("empty-operators", "    expect: {'/type': {}}", "assertion object requires an operator"),
        (
            "object-equality-explicit",
            "    expect: {'/type': {type: string}}",
            "unknown assertion operator type",
        ),
        (
            "nested-wildcard-missing",
            "  - run: [schema, biomcp.problem/2]\n    expect: {'/properties/record/anyOf/*/$ref': [null]}",
            "actual <missing>",
        ),
        ("count-fraction", "    expect: {'/x-extensible-enum': {count: 1.5}}", "count requires a nonnegative integer"),
        (
            "rank-negative",
            "    expect: {'/x-extensible-enum': {rank_of: {hgnc: -1}}}",
            "rank_of requires positive integer ranks",
        ),
        ("count-negative", "    expect: {'/x-extensible-enum': {count: -1}}", "count requires a nonnegative integer"),
        (
            "count-range",
            "    expect: {'/x-extensible-enum': {count: {min: 1}}}",
            "count requires a nonnegative integer",
        ),
        ("rank-shape", "    expect: {'/x-extensible-enum': {rank_of: [hgnc]}}", "rank_of requires a nonempty object"),
        (
            "rank-zero",
            "    expect: {'/x-extensible-enum': {rank_of: {hgnc: 0}}}",
            "rank_of requires positive integer ranks",
        ),
        (
            "rank-range",
            "    expect: {'/x-extensible-enum': {rank_of: {hgnc: '<3'}}}",
            "rank_of requires positive integer ranks",
        ),
        ("includes-type", "    expect: {'/type': {includes: string}}", "includes: expected an array"),
        ("count-type", "    expect: {'/type': {count: 6}}", "count: expected an array"),
        ("rank-type", "    expect: {'/type': {rank_of: {string: 1}}}", "rank_of: expected an array"),
        ("count-wrong", "    expect: {'/x-extensible-enum': {count: 1}}", "count: expected 1, actual"),
        (
            "includes-missing",
            "    expect: {'/x-extensible-enum': {includes: absent}}",
            "includes: expected member \"absent\"",
        ),
        (
            "rank-missing",
            "    expect: {'/x-extensible-enum': {rank_of: {absent: 1}}}",
            "rank_of absent: expected 1, actual <missing>",
        ),
        (
            "rank-wrong",
            "    expect: {'/x-extensible-enum': {rank_of: {pubmed: 1}}}",
            "rank_of pubmed: expected 1, actual",
        ),
        ("wildcard-type", "    expect: {'/type/*': [null]}", "wildcard requires an array"),
        ("wildcard-missing", "    expect: {'/x-extensible-enum/*/missing': null}", "actual <missing>"),
        ("bad-pointer", "    expect: {'enum': []}", "expected a JSON pointer"),
        ("bad-escape", "    expect: {'/~2': []}", "invalid JSON pointer escape"),
        (
            "ceiling-shape",
            "    expect: {'/type': {max_bytes: '8'}}",
            "max_bytes requires a nonnegative integer",
        ),
        (
            "validate-all-steps",
            "    expect: {'/type': integer}\n  - run: [schema, biomcp.source/1]\n    expect: {'/type': {unknown: null}}",
            "unknown assertion operator unknown",
        ),
    ]
    .into_iter()
    .chain(article_cases())
    .collect()
}

fn article_cases() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("follow-absent", "  - follow: page", "follow action absent"),
        (
            "walk-non-page",
            "    walk: {items: /x-extensible-enum, max_pages: 2}",
            "walk requires page offset",
        ),
        (
            "nonincreasing-wrong-order",
            "  - run: [schema, biomcp.article-search/3]\n    expect: {'/properties/limit/maximum': {nonincreasing: true}}",
            "nonincreasing: expected an array",
        ),
        (
            "unique-duplicate",
            "  - run: [schema, biomcp.gene-resolution/7]\n    expect: {'/oneOf/*/properties/resolution/type': {unique: true}}",
            "unique: duplicate member",
        ),
        (
            "includes-all-missing",
            "    expect: {'/x-extensible-enum': {includes_all_of: missing-floor.json}}",
            "includes_all_of: missing member",
        ),
        (
            "equals-floor-wrong",
            "    expect: {'/x-extensible-enum': {equals_file: []}}",
            "expected [], actual",
        ),
    ]
}
