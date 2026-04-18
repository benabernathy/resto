use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "resto", about = "A developer-friendly REST client")]
pub struct Cli {
    /// Path to the collection file
    pub file: PathBuf,

    /// Name of the request to run (omits runs all)
    pub requests: Option<String>,

    // Only print the request name and the HTTP status code
    #[arg(short, long)]
    pub quiet: bool,

    // Exit 0 on success, 1 on failure - no output
    #[arg(short = 'Q', long)]
    pub silent: bool,

    #[arg(short, long)]
    pub verbose: bool,
}
