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

## Features

- [ ] parse custom fixed-field log lines
- [ ] filter by timestamp (=, >, <, >=, <=)
- [ ] filter by level (=, !=)
- [ ] filter by source (=, !=)
- [ ] filter by message (CONTAINS)
- [ ] boolean composition: AND, OR, NOT, parentheses
- [ ] print matching lines in full

## Layout

Single file: main.rs holds the Level and LogRecord type definitions, with Level's FromStr/Display and LogRecord's Display implementations. It also declares AnalyzerError and Query as empty stubs — names reserved for the error type and the query AST, carrying no behaviour yet, so they aren't responsibilities.

As it grows it will hold the real error type, parser, and evaluator, at which point the file carries more than one responsibility — see Known debts.

### Known debts

- IN operator dropped (redundant with OR). Revisit if writing level = "WARN" OR level = "ERROR" becomes annoying in practice.

## Project-wide invariants

- Timestamp ordering is lexicographic, valid only while all timestamps carry a literal Z. A non-Z offset breaks this; that's the trigger to revisit and add a date library.
- Grammar is frozen in this README. New syntax goes in known debts, not the tokenizer.
- LogRecord.source is always stored without brackets. [database] in the file → "database" in the struct, everywhere.
