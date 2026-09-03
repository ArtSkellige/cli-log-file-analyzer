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

// Day 6: becomes a real parsed AST type.
pub struct Query;
// Day 3: becomes the project's error type.
pub struct AnalyzerError;

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
}
