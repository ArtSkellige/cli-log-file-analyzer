Day 1 — Setup, harness, contracts
Scaffold, repo, remote. Fill the README Toolchain block with commands you actually ran, not ones you expect to work. One trivially passing test. Verify the fixture matches the README's log-line format — both the example template and the field rules below it. Sketch type names only: LogRecord, Level, Query, and the error type.

Day 2 — Level and LogRecord
Structs and traits, directly. Level gets FromStr and Display; assert the exact serialized spelling first (ERROR, not Error) — the README already claims it, the test is what makes the claim checkable. LogRecord gets its output Display. Exact strings, test-first.

Day 3 — Line parsing and the error type
One valid line → LogRecord. Then a malformed line → your error, carrying the line number. The error type's shape is derived from what the test needs to match on, not guessed. Decide the malformed-line policy here (skip-and-count vs abort) and test it.

Day 4 — Reading the file as an iterator
Lightest day; it's your slack. Tests take &str or impl BufRead, never a path — that's the seam the persistence day would have forced, showing up here instead. Take it. Output is a lazy iterator of Result<LogRecord>. This is the iterators day.

Day 5 — Tokenizer
No library, so no spike needed — full test-first. Query string → Vec<Token>. The frozen grammar means a small fixed token set.

Day 6 — Parser → Query
Tokens → AST, recursive descent. The error cases are the real work: unknown column, missing operand, trailing garbage, > applied to a column that doesn't order.

Day 7 — Evaluation + CLI wiring
Query + LogRecord → bool. Walk the AST, evaluate each predicate against the struct fields, short-circuit AND/OR correctly. Output is matching lines printed in full.
Then the one place test-first bends: argument parsing. Spike std::env::args, delete the spike, write tests for how your two arguments parse (positional file path, query string), reimplement. No clap.

Day 8 — The hard idea: traits vs enum for the WHERE tree
Replaces the skeleton's concurrency day. Make AND/OR composition explicit and confront Box<dyn Predicate> vs a recursive enum. Both are correct; they have different costs, and picking one deliberately is the lesson. You can assert the observable contract — which records match — test-first, so this day doesn't bend.

Day 9 — Coverage audit + clippy + refactor (skeleton day 7, unchanged)

Day 10 — Docs + final review (skeleton days 8 and 9, merged)
This is the compression point, and I'm naming it rather than pretending it isn't. Rust doctests are real tests and deserve real time; if day 10 gets tight, cut the top-to-bottom re-read, not the doctests.
