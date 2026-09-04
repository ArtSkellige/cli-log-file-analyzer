# cli-log-file-analyzer

Command-line tool that filters a log file using a mini-SQL query string.

## Toolchain

- Language / runtime: `Rust`
- Package manager: `cargo`

```text
build   cargo build
test    cargo test
watch   cargo watch -x check     (needs: cargo install cargo-watch, one-time)
lint    cargo clippy     (needs: rustup component add clippy, one-time)
docs    cargo doc --no-deps
```

## Data

Log line format:

```text
2021-02-09T11:40:59Z ERROR [database] connection refused after 3 retries
```

Query grammar (frozen):

```text
query        = expr
expr         = and_expr ( "OR" and_expr )*
and_expr     = not_expr ( "AND" not_expr )*
not_expr     = "NOT" not_expr | atom
atom         = comparison | "(" expr ")"
comparison   = "timestamp" timestamp_op string
             | "level"     eq_op         string
             | "source"    eq_op         string
             | "message"   "CONTAINS"    string
timestamp_op = "=" | ">" | "<" | ">=" | "<="
eq_op        = "=" | "!="
string       = '"' [^"]* '"'
```

Field rules:

- timestamp — RFC3339, literal Z suffix, always 20 chars. Lexicographic ordering is valid and is an invariant: it breaks the moment a non-Z offset is accepted.
- level — exact spellings: ERROR, WARN, INFO, DEBUG. Severity order: DEBUG < INFO < WARN < ERROR. Comparisons use this order, not alphabetical.
- source — stored without brackets. [database] in the file → "database" in the struct.
- message — everything after `] `. May contain spaces.
- source — stored without brackets. [database] in the file → "database" in
  the struct. May not be empty or whitespace-only once brackets are
  stripped — a line with `[]` or `[ ]` is rejected as malformed.

## Features

- [x] parse custom fixed-field log lines
- [ ] filter by timestamp (=, >, <, >=, <=)
- [ ] filter by level (=, !=)
- [ ] filter by source (=, !=)
- [ ] filter by message (CONTAINS)
- [ ] boolean composition: AND, OR, NOT, parentheses
- [ ] print matching lines in full

## Layout

Single file: main.rs holds the Level and LogRecord type definitions (Level's
FromStr/Display, LogRecord's Display), LogRecord::parse (turns one log line
into a LogRecord or an AnalyzerError carrying the line number), and the real
AnalyzerError/ErrorKind types. Query remains an empty stub — a name reserved
for the query AST, carrying no behaviour yet.

It already carries more than one responsibility — types, line-parsing, and
the error type — see Known debts.

### Known debts

- IN operator dropped (redundant with OR). Revisit if writing level = "WARN" OR level = "ERROR" becomes annoying in practice.
- `main.rs` holds data types, a real error type, line-parsing logic, a query
  stub, and the full test suite — more than one responsibility. Seam: types +
  Display impls, the error type, the parser, and tests are each a plausible
  own module. Left alone because Day 5/6 (tokenizer, query parser) will reveal
  the real module boundaries — splitting now risks guessing wrong and
  resplitting later. Revisit when Day 5's tokenizer lands, or when the file
  nears the profile's 500-line default (283 today), whichever comes first.
- `LogRecord::parse`'s malformed-field detection mixes two strategies:
  counting how many pieces `head.splitn(3, ' ')` produced (structural), and
  pattern-matching `level_str` for a stray `[`/`]` to detect a skipped level
  (shape-based). Seam: locate the source's literal `[` directly instead of
  inferring its position from the second space, which would remove the need
  for the shape check. Left alone because every constructed malformed input
  parses correctly today and the heuristic hasn't misfired yet. Revisit if a
  third shape-based check gets added, or a case surfaces where this one
  misclassifies a line.

## Project-wide invariants

- Timestamp ordering is lexicographic, valid only while all timestamps carry a literal Z. A non-Z offset breaks this; that's the trigger to revisit and add a date library.
- Grammar is frozen in this README. New syntax goes in known debts, not the tokenizer.
- LogRecord.source is always stored without brackets. [database] in the file → "database" in the struct, everywhere.
- When a line fails more than one structural check, LogRecord::parse reports
  whichever check runs first in the function body, not the most severe or an
  exhaustive list — check order is significant, not incidental.
