use std::io::BufRead;

mod parser;
mod token;
#[derive(Debug, PartialEq)]
pub struct LogRecord {
    pub timestamp: String,
    pub level: Level,
    pub source: String,
    pub message: String,
}

impl std::fmt::Display for LogRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} [{}] {}",
            self.timestamp, self.level, self.source, self.message
        )
    }
}

impl LogRecord {
    pub fn parse(line: &str, line_number: usize) -> Result<Self, AnalyzerError> {
        let bracket_space_idx = line.find("] ").ok_or(AnalyzerError {
            line: line_number,
            kind: ErrorKind::MissingSourceBrackets,
        })?;

        let head = &line[..=bracket_space_idx];
        let message = &line[bracket_space_idx + 2..];

        let pieces: Vec<&str> = head.splitn(3, ' ').collect();

        let timestamp_str = *pieces.first().expect(
            "head always contains at least the source-bracket byte once \"] \" has been found",
        );

        let level_str = *pieces.get(1).ok_or(AnalyzerError {
            line: line_number,
            kind: ErrorKind::MissingLevel,
        })?;

        // A missing level shifts the bracketed source into this slot instead — either the
        // whole thing ("[database]") or, if the source is also missing its own "[", just
        // the tail of it ("source]"). Either shape means the level field itself was never
        // there; it's not that this string is an invalid level.
        if level_str.starts_with('[') || level_str.ends_with(']') {
            return Err(AnalyzerError {
                line: line_number,
                kind: ErrorKind::MissingLevel,
            });
        }

        let source_raw = *pieces.get(2).ok_or(AnalyzerError {
            line: line_number,
            kind: ErrorKind::MissingSource,
        })?;

        if !source_raw.starts_with('[') {
            return Err(AnalyzerError {
                line: line_number,
                kind: ErrorKind::MissingSourceOpenBracket,
            });
        }

        let inner_source = &source_raw[1..source_raw.len() - 1];

        if inner_source.trim().is_empty() {
            return Err(AnalyzerError {
                line: line_number,
                kind: ErrorKind::MissingSource,
            });
        }

        let timestamp = timestamp_str.to_string();

        let level = level_str.parse::<Level>().map_err(|_| AnalyzerError {
            line: line_number,
            kind: ErrorKind::InvalidLevel(level_str.to_string()),
        })?;

        let source = inner_source.to_string();

        Ok(LogRecord {
            timestamp,
            level,
            source,
            message: message.to_string(),
        })
    }
}

#[derive(Debug, PartialEq)]

pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
        };
        write!(f, "{display_str}")
    }
}

impl std::str::FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ERROR" => Ok(Self::Error),
            "WARN" => Ok(Self::Warn),
            "INFO" => Ok(Self::Info),
            "DEBUG" => Ok(Self::Debug),
            _ => Err(format!("invalid level: {s}")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AnalyzerError {
    pub line: usize,
    pub kind: ErrorKind,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    MissingLevel,
    MissingSource,
    MissingSourceBrackets,
    MissingSourceOpenBracket,
    InvalidLevel(String),
}

#[derive(Debug)]
pub enum LogError {
    Io(std::io::Error),
    Parse(AnalyzerError),
}

// Derived PartialEq is impossible here — io::Error doesn't implement it — so
// Io variants compare by ErrorKind alone. Two Io errors with the same kind but
// different messages are treated as equal.
impl PartialEq for LogError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Parse(a), Self::Parse(b)) => a == b,
            (Self::Io(a), Self::Io(b)) => a.kind() == b.kind(),

            _ => false,
        }
    }
}

pub struct LogRecords<R: BufRead> {
    reader: R,
    next_line: usize,
}

impl<R: BufRead> LogRecords<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            // 1-indexed to match AnalyzerError.line's convention — the first line read is line 1.
            next_line: 1,
        }
    }
}

impl<R: BufRead> Iterator for LogRecords<R> {
    type Item = Result<LogRecord, LogError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line_buffer = String::new();

        match self.reader.read_line(&mut line_buffer) {
            Err(e) => Some(Err(LogError::Io(e))),
            Ok(0) => None,

            Ok(_) => {
                // read_line keeps the line terminator (unlike .lines()); if left in, it
                // ends up glued onto the end of LogRecord::message.
                let trimmed_line = line_buffer.trim_end_matches(['\n', '\r']);
                let line_number = self.next_line;
                self.next_line += 1;

                let result = LogRecord::parse(trimmed_line, line_number).map_err(LogError::Parse);

                Some(result)
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn harness_runs() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn error_level_displays_as_error() {
        assert_eq!(Level::Error.to_string(), "ERROR");
    }

    #[test]
    fn warn_level_displays_as_warn() {
        assert_eq!(Level::Warn.to_string(), "WARN");
    }

    #[test]
    fn info_level_displays_as_info() {
        assert_eq!(Level::Info.to_string(), "INFO");
    }

    #[test]
    fn debug_level_displays_as_debug() {
        assert_eq!(Level::Debug.to_string(), "DEBUG");
    }

    #[test]
    fn error_string_parses_to_error_level() {
        assert_eq!("ERROR".parse::<Level>(), Ok(Level::Error));
    }

    #[test]
    fn warn_string_parses_to_warn_level() {
        assert_eq!("WARN".parse::<Level>(), Ok(Level::Warn));
    }

    #[test]
    fn info_string_parses_to_info_level() {
        assert_eq!("INFO".parse::<Level>(), Ok(Level::Info));
    }

    #[test]
    fn debug_string_parses_to_debug_level() {
        assert_eq!("DEBUG".parse::<Level>(), Ok(Level::Debug));
    }

    #[test]
    fn log_record_displays_as_original_line() {
        let record = LogRecord {
            timestamp: "2021-02-09T11:40:59Z".to_string(),
            level: Level::Error,
            source: "database".to_string(),
            message: "connection refused after 3 retries".to_string(),
        };
        assert_eq!(
            record.to_string(),
            "2021-02-09T11:40:59Z ERROR [database] connection refused after 3 retries"
        );
    }

    #[test]
    fn valid_line_parses_to_log_record() {
        let line = "2021-02-09T11:40:59Z ERROR [database] connection refused after 3 retries";
        let expected = LogRecord {
            timestamp: "2021-02-09T11:40:59Z".to_string(),
            level: Level::Error,
            source: "database".to_string(),
            message: "connection refused after 3 retries".to_string(),
        };
        assert_eq!(LogRecord::parse(line, 1), Ok(expected));
    }

    #[test]
    fn missing_closing_bracket_fails_parsing() {
        let line = "2021-02-09T11:40:59Z ERROR [database connection refused after 3 retries";

        assert_eq!(
            LogRecord::parse(line, 42),
            Err(AnalyzerError {
                line: 42,
                kind: ErrorKind::MissingSourceBrackets,
            })
        );
    }

    #[test]
    fn missing_open_bracket_fails_parsing() {
        let line = "2021-02-09T11:40:59Z ERROR database] connection refused after 3 retries";

        assert_eq!(
            LogRecord::parse(line, 7),
            Err(AnalyzerError {
                line: 7,
                kind: ErrorKind::MissingSourceOpenBracket,
            })
        );
    }

    #[test]
    fn missing_level_fails_parsing() {
        let line = "2021-02-09T11:40:59Z [database] connection refused after 3 retries";

        assert_eq!(
            LogRecord::parse(line, 12),
            Err(AnalyzerError {
                line: 12,
                kind: ErrorKind::MissingLevel,
            })
        );
    }

    #[test]
    fn missing_source_fails_parsing() {
        let line = "2021-02-09T11:40:59Z ERROR [] connection refused after 3 retries";

        assert_eq!(
            LogRecord::parse(line, 15),
            Err(AnalyzerError {
                line: 15,
                kind: ErrorKind::MissingSource,
            })
        );
    }

    #[test]
    fn invalid_level_fails_parsing() {
        let line = "2021-02-09T11:40:59Z NOTICE [database] connection refused after 3 retries";

        assert_eq!(
            LogRecord::parse(line, 99),
            Err(AnalyzerError {
                line: 99,
                kind: ErrorKind::InvalidLevel("NOTICE".to_string()),
            })
        );
    }

    #[test]
    fn whitespace_only_source_fails_parsing() {
        let line = "2021-02-09T11:40:59Z ERROR [   ] connection refused after 3 retries";

        assert_eq!(
            LogRecord::parse(line, 19),
            Err(AnalyzerError {
                line: 19,
                kind: ErrorKind::MissingSource,
            })
        );
    }

    #[test]
    fn missing_level_ends_with_bracket_fails_parsing() {
        let line = "2021-02-09T11:40:59Z source] connection refused after 3 retries";

        assert_eq!(
            LogRecord::parse(line, 23),
            Err(AnalyzerError {
                line: 23,
                kind: ErrorKind::MissingLevel,
            })
        )
    }

    #[test]
    fn stream_parser_assigns_correct_line_numbers_to_errors() {
        let data = "2021-02-09T11:40:59Z ERROR [database] fine line\n2021-02-09T11:40:59Z [database] missing level here";
        let reader = std::io::Cursor::new(data);

        let mut stream = LogRecords::new(reader);

        assert!(stream.next().unwrap().is_ok());

        assert_eq!(
            stream.next().unwrap(),
            Err(LogError::Parse(AnalyzerError {
                line: 2,
                kind: ErrorKind::MissingLevel,
            }))
        );
    }

    #[test]
    fn empty_input_immediately_returns_none() {
        let data = "";
        let reader = std::io::Cursor::new(data);
        let mut stream = LogRecords::new(reader);

        assert_eq!(stream.next(), None);
    }

    #[test]
    fn stream_stops_returning_items_after_eof() {
        let data = "2021-02-09T11:40:59Z INFO [server] single line";
        let reader = std::io::Cursor::new(data);
        let mut stream = LogRecords::new(reader);

        assert!(stream.next().unwrap().is_ok());
        assert_eq!(stream.next(), None);
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn io_error_propagates_properly() {
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

        let mut stream = LogRecords::new(FailingReader);
        let expected_err =
            LogError::Io(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));

        assert_eq!(stream.next().unwrap(), Err(expected_err));
    }
}
