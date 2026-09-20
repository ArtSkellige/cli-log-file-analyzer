use crate::evaluator::evaluate;
use crate::parser::{ParseError, parse};
use crate::token::{TokenizeError, tokenize};
use crate::{LogError, LogRecords};
use std::fmt;
use std::io::{BufRead, Write};

#[derive(Debug)]
pub enum CliError {
    Tokenize(TokenizeError),
    Parse(ParseError),
    Io(std::io::Error),
}

// Derived PartialEq is impossible here — io::Error doesn't implement it — so
// Io variants compare by ErrorKind alone. Two Io errors with the same kind but
// different messages are treated as equal.
impl PartialEq for CliError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tokenize(a), Self::Tokenize(b)) => a == b,
            (Self::Parse(a), Self::Parse(b)) => a == b,
            (Self::Io(a), Self::Io(b)) => a.kind() == b.kind(),
            _ => false,
        }
    }
}

// Tokenize/Parse arms use {:?} — TokenizeError and ParseError don't implement
// Display. Giving them one is a reasonable future step, not done here.
impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Tokenize(err) => write!(f, "could not tokenize query: {err:?}"),
            CliError::Parse(err) => write!(f, "could not parse query: {err:?}"),
            CliError::Io(err) => write!(f, "I/O error while reading log file: {err}"),
        }
    }
}

pub fn run(
    query: &str,
    log_reader: impl BufRead,
    mut stdout: impl Write,
    mut stderr: impl Write,
) -> Result<(), CliError> {
    let tokens = tokenize(query).map_err(CliError::Tokenize)?;

    let query = parse(&tokens).map_err(CliError::Parse)?;

    let records = LogRecords::new(log_reader);
    for record_result in records {
        match record_result {
            Ok(record) => {
                if evaluate(&query, &record) {
                    writeln!(stdout, "{}", record).map_err(CliError::Io)?;
                }
            }
            Err(LogError::Parse(e)) => {
                writeln!(
                    stderr,
                    "warning: line {}: malformed log line ({:?}), skipped",
                    e.line, e.kind
                )
                .map_err(CliError::Io)?;
            }
            Err(LogError::Io(e)) => {
                return Err(CliError::Io(e));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn matching_line_is_written_to_stdout() {
        let line = "2021-02-09T11:40:59Z ERROR [database] connection refused after 3 retries";
        let query = "level = \"ERROR\"";

        let log_reader = Cursor::new(line);
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let result = run(query, log_reader, &mut stdout_buf, &mut stderr_buf);

        assert_eq!(result, Ok(()));

        let expected_stdout = format!("{}\n", line);
        let actual_stdout = String::from_utf8(stdout_buf).expect("stdout should be valid UTF-8");
        assert_eq!(actual_stdout, expected_stdout);

        assert!(stderr_buf.is_empty(), "Expected stderr to be empty");
    }

    #[test]
    fn non_matching_line_is_not_written_to_stdout() {
        let line = "2021-02-09T11:40:59Z ERROR [database] connection refused after 3 retries";
        let query = "level = \"WARN\"";

        let log_reader = Cursor::new(line);
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let result = run(query, log_reader, &mut stdout_buf, &mut stderr_buf);

        assert_eq!(result, Ok(()));
        assert!(
            stdout_buf.is_empty(),
            "Expected stdout to be empty for non-matching line"
        );
        assert!(stderr_buf.is_empty(), "Expected stderr to be empty");
    }

    #[test]
    fn unterminated_string_query_returns_tokenize_error() {
        let query = "level = \"ERROR";
        let log_reader = Cursor::new("");
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let result = run(query, log_reader, &mut stdout_buf, &mut stderr_buf);

        assert_eq!(
            result,
            Err(CliError::Tokenize(TokenizeError::UnterminatedString))
        );
        assert!(
            stdout_buf.is_empty(),
            "Expected stdout to be empty on query error"
        );
        assert!(
            stderr_buf.is_empty(),
            "Expected stderr to be empty on query error"
        );
    }

    #[test]
    fn unknown_column_query_returns_parse_error() {
        let query = "foo = \"bar\"";
        let log_reader = Cursor::new("");
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let result = run(query, log_reader, &mut stdout_buf, &mut stderr_buf);

        assert_eq!(
            result,
            Err(CliError::Parse(ParseError::UnknownColumn(
                "foo".to_string()
            )))
        );
        assert!(
            stdout_buf.is_empty(),
            "Expected stdout to be empty on query error"
        );
        assert!(
            stderr_buf.is_empty(),
            "Expected stderr to be empty on query error"
        );
    }

    #[test]
    fn malformed_log_line_is_skipped_and_warned_to_stderr() {
        let data = "not a valid log line\n2021-02-09T11:40:59Z ERROR [database] connection refused after 3 retries";
        let query = "level = \"ERROR\"";

        let log_reader = Cursor::new(data);
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let result = run(query, log_reader, &mut stdout_buf, &mut stderr_buf);

        assert_eq!(result, Ok(()));

        let expected_stdout =
            "2021-02-09T11:40:59Z ERROR [database] connection refused after 3 retries\n";
        let actual_stdout = String::from_utf8(stdout_buf).expect("stdout should be valid UTF-8");
        assert_eq!(actual_stdout, expected_stdout);

        let expected_stderr =
            "warning: line 1: malformed log line (MissingSourceBrackets), skipped\n";
        let actual_stderr = String::from_utf8(stderr_buf).expect("stderr should be valid UTF-8");
        assert_eq!(actual_stderr, expected_stderr);
    }

    #[test]
    fn io_error_stops_loop_early_and_returns_error() {
        struct FailingReader;
        impl std::io::Read for FailingReader {
            fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "simulated network drop",
                ))
            }
        }
        impl BufRead for FailingReader {
            fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "simulated network drop",
                ))
            }
            fn consume(&mut self, _amt: usize) {}
        }

        let query = "level = \"ERROR\"";
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let result = run(query, FailingReader, &mut stdout_buf, &mut stderr_buf);

        let expected_err =
            CliError::Io(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
        assert_eq!(result, Err(expected_err));
        assert!(
            stdout_buf.is_empty(),
            "Expected stdout to be empty on fatal IO error"
        );
        assert!(
            stderr_buf.is_empty(),
            "Expected stderr to be empty on fatal IO error"
        );
    }

    #[test]
    fn cross_variant_comparison_returns_false() {
        let tokenize_err = CliError::Tokenize(TokenizeError::UnterminatedString);
        let parse_err = CliError::Parse(ParseError::ExpectedColumn);

        assert_ne!(tokenize_err, parse_err);
    }

    #[test]
    fn tokenize_error_displays_correctly() {
        assert_eq!(
            CliError::Tokenize(TokenizeError::UnterminatedString).to_string(),
            "could not tokenize query: UnterminatedString"
        );
    }

    #[test]
    fn parse_error_displays_correctly() {
        assert_eq!(
            CliError::Parse(ParseError::ExpectedColumn).to_string(),
            "could not parse query: ExpectedColumn"
        );
    }

    #[test]
    fn io_error_displays_with_wrapped_message() {
        let inner = std::io::Error::from(std::io::ErrorKind::NotFound);
        let inner_msg = inner.to_string();
        assert_eq!(
            CliError::Io(inner).to_string(),
            format!("I/O error while reading log file: {inner_msg}")
        );
    }
}
