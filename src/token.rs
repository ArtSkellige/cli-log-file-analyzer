#[derive(Debug, PartialEq)]
pub enum Token {
    // Covers both grammar keywords (AND/OR/NOT/CONTAINS) and column names
    // (timestamp/level/source/message) — the tokenizer doesn't validate which
    // words are meaningful. Unknown-column/keyword detection is a Day 6
    // parser error, not a tokenizer one.
    Word(String),
    StringLit(String),
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    LParen,
    RParen,
}

// No line number, unlike AnalyzerError — a query string is one line, not a file.
#[derive(Debug, PartialEq)]
pub enum TokenizeError {
    UnterminatedString,
    Unexpected(char),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '>' {
            chars.next();
            if chars.next_if(|&next_c| next_c == '=').is_some() {
                tokens.push(Token::Gte);
            } else {
                tokens.push(Token::Gt);
            }
        } else if c == '<' {
            chars.next();
            if chars.next_if(|&next_c| next_c == '=').is_some() {
                tokens.push(Token::Lte);
            } else {
                tokens.push(Token::Lt);
            }
        } else if c == '!' {
            let ch = chars.next().unwrap();
            if chars.next_if(|&next_c| next_c == '=').is_some() {
                tokens.push(Token::Ne);
            } else {
                // Unlike `>`/`<`, bare `!` isn't valid anywhere in the grammar — there's
                // no single-char fallback token to push here.
                return Err(TokenizeError::Unexpected(ch));
            }
        } else if c == '=' {
            chars.next();
            tokens.push(Token::Eq);
        } else if c == '(' {
            chars.next();
            tokens.push(Token::LParen);
        } else if c == ')' {
            chars.next();
            tokens.push(Token::RParen);
        } else if c == '"' {
            chars.next();
            let mut contents = String::new();
            let mut closed = false;

            for next_c in chars.by_ref() {
                if next_c == '"' {
                    closed = true;
                    break;
                } else {
                    contents.push(next_c);
                }
            }
            if !closed {
                return Err(TokenizeError::UnterminatedString);
            }
            tokens.push(Token::StringLit(contents));
        } else if c.is_alphabetic() {
            let mut word = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c.is_alphabetic() {
                    word.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(Token::Word(word));
        } else if c.is_whitespace() {
            chars.next();
        } else {
            return Err(TokenizeError::Unexpected(chars.next().unwrap()));
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_word_tokenizes_to_word() {
        assert_eq!(
            tokenize("timestamp"),
            Ok(vec![Token::Word("timestamp".to_string())])
        );
    }

    #[test]
    fn whitespace_between_words_is_skipped() {
        assert_eq!(
            tokenize("timestamp level"),
            Ok(vec![
                Token::Word("timestamp".to_string()),
                Token::Word("level".to_string())
            ])
        );
    }

    #[test]
    fn unrecognized_characters_is_an_error() {
        assert_eq!(tokenize("#"), Err(TokenizeError::Unexpected('#')));
    }

    #[test]
    fn equals_sign_tokenizes_to_eq() {
        assert_eq!(tokenize("="), Ok(vec![Token::Eq]));
    }

    #[test]
    fn not_equals_tokenizes_to_ne() {
        assert_eq!(tokenize("!="), Ok(vec![Token::Ne]));
    }

    #[test]
    fn bare_exclamation_is_an_error() {
        assert_eq!(tokenize("!"), Err(TokenizeError::Unexpected('!')));
    }

    #[test]
    fn greater_than_tokenizes_to_gt() {
        assert_eq!(tokenize(">"), Ok(vec![Token::Gt]));
    }

    #[test]
    fn greater_than_or_equal_tokenizes_to_gte() {
        assert_eq!(tokenize(">="), Ok(vec![Token::Gte]));
    }

    #[test]
    fn less_than_tokenizes_to_lt() {
        assert_eq!(tokenize("<"), Ok(vec![Token::Lt]));
    }

    #[test]
    fn less_than_or_equal_tokenizes_to_lte() {
        assert_eq!(tokenize("<="), Ok(vec![Token::Lte]));
    }

    #[test]
    fn left_paren_tokenizes_to_lparen() {
        assert_eq!(tokenize("("), Ok(vec![Token::LParen]));
    }

    #[test]
    fn right_paren_tokenizes_to_rparen() {
        assert_eq!(tokenize(")"), Ok(vec![Token::RParen]));
    }

    #[test]
    fn quoted_string_tokenizes_to_string_lit() {
        assert_eq!(
            tokenize("\"foo\""),
            Ok(vec![Token::StringLit("foo".to_string())])
        );
    }

    #[test]
    fn unterminated_string_is_an_error() {
        assert_eq!(tokenize("\"foo"), Err(TokenizeError::UnterminatedString));
    }
}
