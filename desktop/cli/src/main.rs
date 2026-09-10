use clap::Parser;
use lanesra_cli::cli::Cli;

fn main() {
    let cli = Cli::parse();
    std::process::exit(lanesra_cli::run(cli));
}
