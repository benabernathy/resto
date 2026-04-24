use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "resto", about = "A developer-friendly REST client")]
pub struct Cli {
    /// Path to the collection file
    pub file: PathBuf,

    /// Name of the request to run (omits runs all)
    pub requests: Option<String>,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Normal)]
    pub output: OutputFormat,
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
