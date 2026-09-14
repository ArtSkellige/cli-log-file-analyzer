#[derive(Debug, PartialEq)]
pub(crate) struct Args {
    pub(crate) file_path: String,
    pub(crate) query: String,
}

#[derive(Debug, PartialEq)]
pub(crate) enum ArgsError {
    MissingFilePath,
    MissingQuery,
    TooManyArguments(usize),
}

pub(crate) fn parse_args(args: &[String]) -> Result<Args, ArgsError> {
    match args.len() {
        2 => Ok(Args {
            file_path: args[0].clone(),
            query: args[1].clone(),
        }),
        0 => Err(ArgsError::MissingFilePath),
        1 => Err(ArgsError::MissingQuery),
        _ => Err(ArgsError::TooManyArguments(args.len())),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn exact_two_arguments_returns_ok_args() {
        let input_args = vec![
            "path/to/log_file.log".to_string(),
            "level = \"ERROR\" AND source = \"database\"".to_string(),
        ];

        let expected = Args {
            file_path: "path/to/log_file.log".to_string(),
            query: "level = \"ERROR\" AND source = \"database\"".to_string(),
        };

        assert_eq!(parse_args(&input_args), Ok(expected));
    }

    #[test]
    fn empty_arguments_returns_missing_file_path_error() {
        assert_eq!(parse_args(&[]), Err(ArgsError::MissingFilePath));
    }

    #[test]
    fn single_argument_returns_missing_query_error() {
        let input_args = vec!["path/to/log_file.log".to_string()];
        assert_eq!(parse_args(&input_args), Err(ArgsError::MissingQuery));
    }

    #[test]
    fn three_arguments_returns_too_many_arguments_error() {
        let input_args = vec![
            "path/to/log_file.log".to_string(),
            "level = \"ERROR\"".to_string(),
            "extra_arg".to_string(),
        ];
        assert_eq!(parse_args(&input_args), Err(ArgsError::TooManyArguments(3)));
    }

    #[test]
    fn five_arguments_returns_too_many_arguments_error() {
        let input_args = vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
        ];
        assert_eq!(parse_args(&input_args), Err(ArgsError::TooManyArguments(5)));
    }
}
