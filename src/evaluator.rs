use crate::LogRecord;
use crate::parser::{Column, Op, Query};

pub(crate) fn evaluate(query: &Query, record: &LogRecord) -> bool {
    match query {
        Query::Comparison(comp) => match (&comp.column, &comp.op) {
            // Compares via Level's Display string rather than parsing comp.value into a Level — evaluate has no error channel to report what an invalid level literal would mean.
            (Column::Level, Op::Eq) => record.level.to_string() == comp.value,

            (Column::Level, Op::Ne) => record.level.to_string() != comp.value,
            (Column::Source, Op::Eq) => record.source == comp.value,
            (Column::Source, Op::Ne) => record.source != comp.value,
            (Column::Message, Op::Contains) => record.message.contains(&comp.value),
            (Column::Timestamp, Op::Eq | Op::Gt | Op::Gte | Op::Lt | Op::Lte) => match comp.op {
                Op::Eq => record.timestamp == comp.value,
                Op::Gt => record.timestamp > comp.value,
                Op::Gte => record.timestamp >= comp.value,
                Op::Lt => record.timestamp < comp.value,
                Op::Lte => record.timestamp <= comp.value,
                // Outer match already narrowed comp.op to these five variants.
                _ => unreachable!(),
            },

            // Every arm below assumes parser::is_valid_op_for_column already rejected this
            // (Column, Op) combination at parse time — see the invariant in README.md.
            (Column::Level, Op::Gt | Op::Gte | Op::Lt | Op::Lte | Op::Contains) => unreachable!(),
            (Column::Source, Op::Gt | Op::Gte | Op::Lt | Op::Lte | Op::Contains) => unreachable!(),
            (Column::Message, Op::Eq | Op::Ne | Op::Gt | Op::Gte | Op::Lt | Op::Lte) => {
                unreachable!()
            }
            (Column::Timestamp, Op::Ne | Op::Contains) => unreachable!(),
        },
        Query::And(left, right) => evaluate(left, record) && evaluate(right, record),
        Query::Or(left, right) => evaluate(left, record) || evaluate(right, record),
        Query::Not(inner) => !evaluate(inner, record),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Level;
    use crate::parser::Comparison;

    #[test]
    fn single_comparison_level_eq_error_is_true() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Error,
            source: String::new(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Level,
            op: Op::Eq,
            value: "ERROR".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_level_eq_error_mismatch_is_false() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Warn,
            source: String::new(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Level,
            op: Op::Eq,
            value: "ERROR".to_string(),
        });

        assert!(!evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_level_ne_error_is_true_for_warn() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Warn,
            source: String::new(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Level,
            op: Op::Ne,
            value: "ERROR".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_source_eq_database_is_true() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Info,
            source: "database".to_string(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Source,
            op: Op::Eq,
            value: "database".to_string(),
        });

        assert!(evaluate(&query, &record))
    }

    #[test]
    fn single_comparison_source_ne_auth_is_true_for_database() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Info,
            source: "database".to_string(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Source,
            op: Op::Ne,
            value: "auth".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_message_contains_refused_is_true() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Error,
            source: String::new(),
            message: "connection refused after 3 retries".to_string(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Message,
            op: Op::Contains,
            value: "refused".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_timestamp_eq_matches_exactly() {
        let record = LogRecord {
            timestamp: "2021-02-09T11:40:12Z".to_string(),
            level: Level::Info,
            source: String::new(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Eq,
            value: "2021-02-09T11:40:12Z".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_timestamp_gt_matches_later_time() {
        let record = LogRecord {
            timestamp: "2021-02-09T11:41:00Z".to_string(),
            level: Level::Info,
            source: String::new(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Gt,
            value: "2021-02-09T11:40:12Z".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_timestamp_gte_matches_exact_boundary() {
        let record = LogRecord {
            timestamp: "2021-02-09T11:40:12Z".to_string(),
            level: Level::Info,
            source: String::new(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Gte,
            value: "2021-02-09T11:40:12Z".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_timestamp_lt_matches_earlier_time() {
        let record = LogRecord {
            timestamp: "2021-02-09T11:39:00Z".to_string(),
            level: Level::Info,
            source: String::new(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Lt,
            value: "2021-02-09T11:40:12Z".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn single_comparison_timestamp_lte_matches_exact_boundary() {
        let record = LogRecord {
            timestamp: "2021-02-09T11:40:12Z".to_string(),
            level: Level::Info,
            source: String::new(),
            message: String::new(),
        };

        let query = Query::Comparison(Comparison {
            column: Column::Timestamp,
            op: Op::Lte,
            value: "2021-02-09T11:40:12Z".to_string(),
        });

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn composite_and_expression_is_true_when_both_arms_match() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Error,
            source: "database".to_string(),
            message: String::new(),
        };

        let query = Query::And(
            Box::new(Query::Comparison(Comparison {
                column: Column::Level,
                op: Op::Eq,
                value: "ERROR".to_string(),
            })),
            Box::new(Query::Comparison(Comparison {
                column: Column::Source,
                op: Op::Eq,
                value: "database".to_string(),
            })),
        );

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn composite_and_expression_is_false_when_one_arm_mismatches() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Error,
            source: "database".to_string(),
            message: String::new(),
        };

        let query = Query::And(
            Box::new(Query::Comparison(Comparison {
                column: Column::Level,
                op: Op::Eq,
                value: "ERROR".to_string(),
            })),
            Box::new(Query::Comparison(Comparison {
                column: Column::Source,
                op: Op::Eq,
                value: "auth".to_string(),
            })),
        );

        assert!(!evaluate(&query, &record));
    }

    #[test]
    fn composite_or_expression_is_true_when_one_arm_matches() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Error,
            source: "database".to_string(),
            message: String::new(),
        };

        let query = Query::Or(
            Box::new(Query::Comparison(Comparison {
                column: Column::Source,
                op: Op::Eq,
                value: "auth".to_string(),
            })),
            Box::new(Query::Comparison(Comparison {
                column: Column::Level,
                op: Op::Eq,
                value: "ERROR".to_string(),
            })),
        );

        assert!(evaluate(&query, &record));
    }

    #[test]
    fn composite_or_expression_is_false_when_both_arms_mismatch() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Error,
            source: "database".to_string(),
            message: String::new(),
        };

        let query = Query::Or(
            Box::new(Query::Comparison(Comparison {
                column: Column::Source,
                op: Op::Eq,
                value: "auth".to_string(),
            })),
            Box::new(Query::Comparison(Comparison {
                column: Column::Level,
                op: Op::Eq,
                value: "WARN".to_string(),
            })),
        );

        assert!(!evaluate(&query, &record));
    }

    #[test]
    fn composite_not_expression_inverts_false_inner_to_true() {
        let record = LogRecord {
            timestamp: String::new(),
            level: Level::Info,
            source: "database".to_string(),
            message: String::new(),
        };

        let query = Query::Not(Box::new(Query::Comparison(Comparison {
            column: Column::Source,
            op: Op::Eq,
            value: "auth".to_string(),
        })));

        assert!(evaluate(&query, &record));
    }
}
