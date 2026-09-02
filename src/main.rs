// Day 1: names only. Shapes land Day 2 (Level, LogRecord),
// Day 3 (error type), Day 6 (Query).
pub struct LogRecord;
pub struct Level;
pub struct Query;
pub struct AnalyzerError;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    #[test]
    fn harness_runs() {
        assert_eq!(2 + 2, 4);
    }
}
