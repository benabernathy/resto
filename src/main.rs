use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

mod cli;
use cli::Cli;

mod interpolate;

mod collection;
use collection::{RequestFile, load_requests};

mod executor;
use executor::execute_request;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let content = fs::read_to_string(&cli.file)
        .with_context(|| format!("could not read {}", cli.file.display()))?;

    let rf: RequestFile = toml::from_str(&content)
        .with_context(|| format!("could not parse {}", cli.file.display()))?;

    for name in rf.requests.keys() {
        println!("{}", name);
    }

    let requests = load_requests(&rf, cli.requests.as_deref())?;

    for (name, req) in &requests {
        let result = execute_request(name, req, &rf.vars, cli.quiet, cli.silent, cli.verbose);
        if let Err(e) = result {
            if !cli.silent {
                eprintln!("Error: {}", e);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}
