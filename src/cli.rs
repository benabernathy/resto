use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "toad", about = "A developer-friendly REST client")]
pub struct Cli {
    /// Path to the collection file
    pub file: PathBuf,

    /// Name of the request to run (omits runs all)
    pub requests: Option<String>,

    // Optional output format/verbosity
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Normal)]
    pub output: OutputFormat,

    // Only list requests in file
    #[arg(short, long)]
    pub list_requests: bool,

    // Optional profile name to use for vars
    #[arg(short, long)]
    pub profile: Option<String>,
}

#[derive(ValueEnum, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Normal,
    Quiet,
    Silent,
    Verbose,
    ResponseOnly,
    RequestOnly,
}
