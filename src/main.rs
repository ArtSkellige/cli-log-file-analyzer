use cli_log_file_analyzer::args;
use cli_log_file_analyzer::cli;

fn main() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    let parsed_args = match args::parse_args(&raw_args) {
        Ok(parsed_args) => parsed_args,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("usage: cli-log-file-analyzer <file> <query>");
            std::process::exit(1);
        }
    };

    let file = match std::fs::File::open(&parsed_args.file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("error: could not open '{}': {e}", parsed_args.file_path);
            std::process::exit(1);
        }
    };
    let reader = std::io::BufReader::new(file);

    if let Err(e) = cli::run(
        &parsed_args.query,
        reader,
        std::io::stdout(),
        std::io::stderr(),
    ) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
