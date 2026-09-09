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

One line per file that holds a decision. Generated files, lockfiles, vendored
code, and data fixtures are not listed. Line counts are deliberately absent —
they go stale, and they were never the thing that mattered.

Rule: each responsibility is one sentence with no "and". A line that needs an
"and" is a split waiting to happen and stays under Known debts until it lands.

| Path       | Responsibility (one sentence, no "and")                                             | Threshold |
| ---------- | ----------------------------------------------------------------------------------- | --------- |
| `main.rs`  | Models one log line — its types, parsing, and errors — and iterates a file of them. | 500       |
| `token.rs` | Turns a query string into a flat stream of tokens.                                  | 500       |

Every row carries its threshold, including the default.

### Known debts

- IN operator dropped (redundant with OR). Revisit if writing level = "WARN" OR level = "ERROR" becomes annoying in practice.
- `main.rs` holds data types, two error types, line-parsing logic, a
  file-reading iterator, a query stub, and the full test suite — more than
  one responsibility. Seam: types + Display impls, the error types, the
  parser, and the iterator are each a plausible own module (tokenizing was
  already split out into `token.rs` on Day 5, on the same reasoning). Left
  alone because Day 6's parser will reveal whether `parser` belongs with
  `token` in one module or stands alone — splitting now risks guessing that
  boundary wrong. Revisit when Day 6's parser lands, or when the file nears
  its 500-line threshold (210 today), whichever comes first.
- `token.rs`'s `>`, `<`, and `!` branches in `tokenize` share the same
  consume-then-lookahead shape (`next_if(|&c| c == '=')`, branch two ways).
  Seam: a helper parametrized by the two-char token and by what happens
  when no `=` follows. Left alone because that "no match" behavior differs
  in kind — a token for `>`/`<`, an error for `!` — so a helper would need
  a closure or `Result` parameter, more machinery than three five-line
  blocks justify. Revisit if a fourth two-char case appears, or if Day 6
  reveals a shared shape worth generalizing.
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
- LogRecords numbers lines starting at 1, matching AnalyzerError.line's
  convention — the first line read is line 1, never 0.
- LogError::Io compares equal by io::ErrorKind alone, not the underlying
  io::Error's message text — two Io errors of the same kind but different
  messages are treated as equal.
- `token::tokenize` never validates a word's meaning — `Token::Word` covers
  keywords and column names alike. Unknown-column and unknown-keyword
  detection are Day 6 parser errors, not tokenizer ones.
