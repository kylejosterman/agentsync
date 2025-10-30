use agentsync::Cli;
use std::process;

fn main() {
    let args = Cli::parse_args();

    if let Err(e) = agentsync::run(args) {
        #[allow(clippy::print_stderr)]
        {
            eprintln!("Error: {e}");
        }
        process::exit(1);
    }
}
