// Not called from `main` yet — Day 7's CLI wiring is the first real caller. Scoped to
// non-test builds only: the test module below already uses everything, so a
// blanket `expect` would flag itself as unfulfilled under `cargo test`.
#![cfg_attr(not(test), expect(dead_code))]

use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnknownColumn(String),
    InvalidOperatorForColumn { column: Column, op: Op },
    ExpectedOperand,
    TrailingGarbage,
    UnclosedParen,
}

#[derive(Debug, PartialEq)]
pub enum Query {
    Comparison(Comparison),
    And(Box<Query>, Box<Query>),
    Or(Box<Query>, Box<Query>),
    Not(Box<Query>),
}

#[derive(Debug, PartialEq)]
pub struct Comparison {
    column: Column,
    op: Op,
    value: String,
}

#[derive(Debug, PartialEq)]
pub enum Column {
    Timestamp,
    Level,
    Source,
    Message,
}

#[derive(Debug, PartialEq)]
pub enum Op {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn parse_expr(&mut self) -> Result<Query, ParseError> {
        let mut left = self.parse_and_expr()?;

        while let Some(Token::Word(w)) = self.peek() {
            if w == "OR" {
                self.advance();
                let right = self.parse_and_expr()?;
                left = Query::Or(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Query, ParseError> {
        let mut left = self.parse_not_expr()?;

        while let Some(Token::Word(w)) = self.peek() {
            if w == "AND" {
                self.advance();
                let right = self.parse_not_expr()?;
                left = Query::And(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_not_expr(&mut self) -> Result<Query, ParseError> {
        if let Some(Token::Word(w)) = self.peek()
            && w == "NOT"
        {
            self.advance();
            let inner = self.parse_not_expr()?;
            return Ok(Query::Not(Box::new(inner)));
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Query, ParseError> {
        if let Some(Token::LParen) = self.peek() {
            self.advance();
            let inner = self.parse_expr()?;

            return match self.advance() {
                Some(Token::RParen) => Ok(inner),
                _ => Err(ParseError::UnclosedParen),
            };
        }

        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Query, ParseError> {
        let column = match self.advance() {
            Some(Token::Word(col_name)) => match col_name.as_str() {
                "timestamp" => Column::Timestamp,
                "level" => Column::Level,
                "source" => Column::Source,
                "message" => Column::Message,
                _ => return Err(ParseError::UnknownColumn(col_name.clone())),
            },
            _ => todo!("unhandled token shape or length"),
        };

        let op = match self.advance() {
            Some(Token::Eq) => Op::Eq,
            Some(Token::Ne) => Op::Ne,
            Some(Token::Gt) => Op::Gt,
            Some(Token::Gte) => Op::Gte,
            Some(Token::Lt) => Op::Lt,
            Some(Token::Lte) => Op::Lte,
            Some(Token::Word(w)) if w == "CONTAINS" => Op::Contains,
            _ => todo!("unhandled token shape or length"),
        };

        let value = match self.advance() {
            Some(Token::StringLit(val)) => val.clone(),
            None => return Err(ParseError::ExpectedOperand),
            _ => todo!("unhandled token shape or length"),
        };

        if !is_valid_op_for_column(&column, &op) {
            return Err(ParseError::InvalidOperatorForColumn { column, op });
        }

        Ok(Query::Comparison(Comparison { column, op, value }))
    }
}

pub fn parse(tokens: &[Token]) -> Result<Query, ParseError> {
    let mut parser = Parser::new(tokens);
    let query = parser.parse_expr()?;

    if parser.peek().is_some() {
        return Err(ParseError::TrailingGarbage);
    }

    Ok(query)
}

// Transcribed directly from the frozen grammar in README.md's comparison rule —
// each column's operator set there is already complete and fixed, not something
// to discover one test at a time the way ParseError's shape was.
fn is_valid_op_for_column(column: &Column, op: &Op) -> bool {
    match column {
        Column::Timestamp => matches!(op, Op::Eq | Op::Gt | Op::Lt | Op::Gte | Op::Lte),
        Column::Level | Column::Source => matches!(op, Op::Eq | Op::Ne),
        Column::Message => matches!(op, Op::Contains),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_simple_comparison() {
        let tokens = vec![
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Level,
            op: Op::Eq,
            value: "ERROR".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_unknown_column_fails() {
        let tokens = vec![
            Token::Word("frobnicate".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
        ];

        let expected = Err(ParseError::UnknownColumn("frobnicate".into()));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_not_equals_comparison() {
        let tokens = vec![
            Token::Word("level".into()),
            Token::Ne,
            Token::StringLit("ERROR".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Level,
            op: Op::Ne,
            value: "ERROR".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_source_column_comparison() {
        let tokens = vec![
            Token::Word("source".into()),
            Token::Eq,
            Token::StringLit("database".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Source,
            op: Op::Eq,
            value: "database".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_timestamp_column_comparison() {
        let tokens = vec![
            Token::Word("timestamp".into()),
            Token::Eq,
            Token::StringLit("2021-02-09T11:40:59Z".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Eq,
            value: "2021-02-09T11:40:59Z".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_timestamp_greater_than_comparison() {
        let tokens = vec![
            Token::Word("timestamp".into()),
            Token::Gt,
            Token::StringLit("2021-02-09T11:40:59Z".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Gt,
            value: "2021-02-09T11:40:59Z".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_timestamp_greater_than_or_equal_comparison() {
        let tokens = vec![
            Token::Word("timestamp".into()),
            Token::Gte,
            Token::StringLit("2021-02-09T11:40:59Z".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Gte,
            value: "2021-02-09T11:40:59Z".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_timestamp_less_than_comparison() {
        let tokens = vec![
            Token::Word("timestamp".into()),
            Token::Lt,
            Token::StringLit("2021-02-09T11:40:59Z".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Lt,
            value: "2021-02-09T11:40:59Z".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_timestamp_less_than_or_equal_comparison() {
        let tokens = vec![
            Token::Word("timestamp".into()),
            Token::Lte,
            Token::StringLit("2021-02-09T11:40:59Z".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Lte,
            value: "2021-02-09T11:40:59Z".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_message_contains_comparison() {
        let tokens = vec![
            Token::Word("message".into()),
            Token::Word("CONTAINS".into()),
            Token::StringLit("connection refused".into()),
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Message,
            op: Op::Contains,
            value: "connection refused".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_not_unary_operator_modifier() {
        let tokens = vec![
            Token::Word("NOT".into()),
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
        ];

        let expected = Ok(Query::Not(Box::new(Query::Comparison(Comparison {
            column: Column::Level,
            op: Op::Eq,
            value: "ERROR".into(),
        }))));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_ordering_operator_on_eq_only_column_fails() {
        let tokens = vec![
            Token::Word("level".into()),
            Token::Gt,
            Token::StringLit("ERROR".into()),
        ];

        let expected = Err(ParseError::InvalidOperatorForColumn {
            column: Column::Level,
            op: Op::Gt,
        });

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_ordering_operator_on_source_column_fails() {
        let tokens = vec![
            Token::Word("source".into()),
            Token::Gt,
            Token::StringLit("database".into()),
        ];

        let expected = Err(ParseError::InvalidOperatorForColumn {
            column: Column::Source,
            op: Op::Gt,
        });

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_missing_operand_fails() {
        let tokens = vec![Token::Word("level".into()), Token::Eq];

        let expected = Err(ParseError::ExpectedOperand);

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_trailing_garbage_fails() {
        let tokens = vec![
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
            Token::Word("extra".into()),
        ];

        let expected = Err(ParseError::TrailingGarbage);

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_and_binary_operator_joining_comparisons() {
        let tokens = vec![
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
            Token::Word("AND".into()),
            Token::Word("source".into()),
            Token::Eq,
            Token::StringLit("database".into()),
        ];

        let expected = Ok(Query::And(
            Box::new(Query::Comparison(Comparison {
                column: Column::Level,
                op: Op::Eq,
                value: "ERROR".into(),
            })),
            Box::new(Query::Comparison(Comparison {
                column: Column::Source,
                op: Op::Eq,
                value: "database".into(),
            })),
        ));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_or_binary_operator_joining_comparisons() {
        let tokens = vec![
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
            Token::Word("OR".into()),
            Token::Word("source".into()),
            Token::Eq,
            Token::StringLit("database".into()),
        ];

        let expected = Ok(Query::Or(
            Box::new(Query::Comparison(Comparison {
                column: Column::Level,
                op: Op::Eq,
                value: "ERROR".into(),
            })),
            Box::new(Query::Comparison(Comparison {
                column: Column::Source,
                op: Op::Eq,
                value: "database".into(),
            })),
        ));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_parenthesized_comparison() {
        let tokens = vec![
            Token::LParen,
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
            Token::RParen,
        ];

        let expected = Ok(Query::Comparison(Comparison {
            column: Column::Level,
            op: Op::Eq,
            value: "ERROR".into(),
        }));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_parens_group_or_under_not() {
        let tokens = vec![
            Token::Word("NOT".into()),
            Token::LParen,
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
            Token::Word("OR".into()),
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("WARN".into()),
            Token::RParen,
        ];

        let expected = Ok(Query::Not(Box::new(Query::Or(
            Box::new(Query::Comparison(Comparison {
                column: Column::Level,
                op: Op::Eq,
                value: "ERROR".into(),
            })),
            Box::new(Query::Comparison(Comparison {
                column: Column::Level,
                op: Op::Eq,
                value: "WARN".into(),
            })),
        ))));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_unclosed_paren_fails() {
        let tokens = vec![
            Token::LParen,
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
        ];

        let expected = Err(ParseError::UnclosedParen);

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn parse_unknown_column_nested_inside_parens_and_and_fails() {
        let tokens = vec![
            Token::Word("level".into()),
            Token::Eq,
            Token::StringLit("ERROR".into()),
            Token::Word("AND".into()),
            Token::LParen,
            Token::Word("frobnicate".into()),
            Token::Eq,
            Token::StringLit("x".into()),
            Token::RParen,
        ];

        let expected = Err(ParseError::UnknownColumn("frobnicate".into()));

        assert_eq!(parse(&tokens), expected);
    }
}
