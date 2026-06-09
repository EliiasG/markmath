use clap::Parser;
use markmath::{configure, run, CompileMode};
use std::path::{Path, PathBuf};


#[derive(Parser)]
#[command(version, about = "A calculator for markdown")]
struct Cli {
    /// Source markdown document
    #[arg(required_unless_present = "configure")]
    input: Option<PathBuf>,

    /// Output path
    #[arg(required_unless_present = "configure")]
    output: Option<PathBuf>,

    #[arg(long)] live: bool,
    #[arg(long)] no_resolve: bool,

    /// Edit the unit library interactively, then exit
    #[arg(long, conflicts_with_all = ["input", "output", "live", "no_resolve"])]
    configure: bool,
}

fn main() {
    let cli = Cli::parse();
    if cli.configure {
        if let Err(e) = configure() {
            println!("{}", e);
        }
        return;
    }
    let compile_mode = if cli.live {
        CompileMode::Live
    } else if cli.no_resolve {
        CompileMode::NonResolving
    } else {
        CompileMode::Resolving
    };
    if let Err(e) = run(compile_mode, Path::new(&cli.input.expect("always some when !cli.configure")), Path::new(&cli.output.unwrap())) {
        eprintln!("{}", e);
    }
}