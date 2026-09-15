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

## Usage

```text
cli-log-file-analyzer <file> <query>
```

Two positional arguments, in order: the log file's path, then the query
string. No flags, no `--` separator — anything else (zero, one, or three or
more arguments) is rejected before the file is even opened. Wrap the query
in quotes if it contains spaces, which it almost always will.

Any failure — bad arguments, a file that can't be opened, a query that
doesn't tokenize or parse — prints one message to stderr and exits non-zero.
A log line that doesn't parse is not one of these: it's skipped, with a
warning on stderr naming its line number, and the run continues.

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
- [x] filter by timestamp (=, >, <, >=, <=)
- [x] filter by level (=, !=)
- [x] filter by source (=, !=)
- [x] filter by message (CONTAINS)
- [x] boolean composition: AND, OR, NOT, parentheses
- [x] print matching lines in full

## Layout

One line per file that holds a decision. Generated files, lockfiles, vendored
code, and data fixtures are not listed. Line counts are deliberately absent —
they go stale, and they were never the thing that mattered.

Rule: each responsibility is one sentence with no "and". A line that needs an
"and" is a split waiting to happen and stays under Known debts until it lands.

```markdown
| Path           | Responsibility (one sentence, no "and")                                                                     | Threshold                                                                                          |
| -------------- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `main.rs`      | Models one log line — its types, parsing, and errors — iterates a file of them, and is the CLI entry point. | 500                                                                                                |
| `token.rs`     | Turns a query string into a flat stream of tokens.                                                          | 500                                                                                                |
| `parser.rs`    | Turns a token stream into a Query AST.                                                                      | 800 — single cohesive recursive-descent parser, splitting would scatter one algorithm across files |
| `evaluator.rs` | Decides whether a LogRecord satisfies a Query AST.                                                          | 500                                                                                                |
| `args.rs`      | Parses the CLI's two positional arguments.                                                                  | 500                                                                                                |
| `cli.rs`       | Runs a query against a log stream, printing each match.                                                     | 500                                                                                                |
```

Every row carries its threshold, including the default.

### Known debts

- IN operator dropped (redundant with OR). Revisit if writing level = "WARN" OR level = "ERROR" becomes annoying in practice.
- `main.rs` is at 492 of its own 500-line threshold and now holds four
  things: data types, two error types, a file-reading iterator, and — as of
  Day 7's CLI wiring — the binary's entry point plus its error-message
  formatting. Both of this entry's original revisit triggers have now
  fired (nearing the threshold, and a concrete new responsibility landed).
  Seam for the newest piece: move the per-variant message formatting for
  `ArgsError` (`args.rs`) and `CliError` (`cli.rs`) into `Display` impls on
  those types, leaving `fn main()` a real thin shim and making the message
  text testable. The original types/errors/iterator seam is unchanged from
  before Day 7. Left alone for now — revisit at the latest during Day 9's
  clippy/refactor pass, sooner if this file is touched again first.
- `token.rs`'s `>`, `<`, and `!` branches in `tokenize` share the same
  consume-then-lookahead shape (`next_if(|&c| c == '=')`, branch two ways).
  Seam: a helper parametrized by the two-char token and by what happens
  when no `=` follows. Left alone because that "no match" behavior differs
  in kind — a token for `>`/`<`, an error for `!` — so a helper would need
  a closure or `Result` parameter, more machinery than three five-line
  blocks justify. Revisit if a fourth two-char case appears.
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
- Grammar is frozen in this README. New syntax goes in known debts, not the tokenizer or the parser.
- LogRecord.source is always stored without brackets. [database] in the file → "database" in the struct, everywhere.
- When a line fails more than one structural check, LogRecord::parse reports
  whichever check runs first in the function body, not the most severe or an
  exhaustive list — check order is significant, not incidental.
- LogRecords numbers lines starting at 1, matching AnalyzerError.line's
  convention — the first line read is line 1, never 0.
- `LogError::Io` and `CliError::Io` both compare equal by `io::ErrorKind`
  alone, not the underlying `io::Error`'s message text — two `Io` errors
  of the same kind but different messages are treated as equal, for both
  types, for the same reason (`io::Error` has no `PartialEq` of its own).
- `token::tokenize` never validates a word's meaning — `Token::Word` covers
  keywords and column names alike. Unknown-column and unknown-keyword
  detection are parser errors, not tokenizer ones.
- `evaluator::evaluate` assumes `parser::parse` has already rejected any
  `(Column, Op)` combination `is_valid_op_for_column` disallows (e.g.
  `level > "WARN"`) — by the time a `Comparison` reaches `evaluate`, that
  combination cannot occur, so the corresponding match arms are
  `unreachable!()` rather than threading a `Result` through `evaluate` for
  a case that can't happen. Same tension as `Op`'s flat enum from Day 6,
  decided the same way for the same reason: consistency over a type-level
  split that would only encode a constraint the parser already enforces.
- A `LogError::Parse` during a run is a warning, not a failure: `cli::run`
  skips the line, writes a warning naming its line number to stderr, and
  keeps going. `LogError::Io` is the opposite — treated as fatal, since a
  failed read is likely to keep failing — and stops the loop immediately.
- Query is a recursive enum (Comparison/And/Or/Not, Box<Query>), not
  Box<dyn Predicate> — decided deliberately on Day 8 after spiking the
  trait-object alternative outside the repo. Valid as long as two things
  hold: the grammar stays frozen (so there's no case for a trait object's
  real benefit, open extension), and the evaluator tests keep relying on
  Query's derived Debug/PartialEq (a boxed trait object can't derive
  either — dyn Predicate has no Debug impl, and Box<dyn Predicate> has no ==).
  Revisit if either stops being true.
