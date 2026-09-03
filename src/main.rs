#[derive(Debug, PartialEq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
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
}
