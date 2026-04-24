use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

mod cli;
use cli::{Cli, OutputFormat};

mod interpolate;

mod collection;
use collection::{RequestFile, load_requests};

mod executor;
use executor::execute_request;

mod output;
use crate::output::{
    NormalOutput, OutputMode, QuietOutput, RequestOnlyOutput, ResponseOnlyOutput, SilentOutput,
    VerboseOutput,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let content = fs::read_to_string(&cli.file)
        .with_context(|| format!("could not read {}", cli.file.display()))?;

    let rf: RequestFile = toml::from_str(&content)
        .with_context(|| format!("could not parse {}", cli.file.display()))?;

    let requests = load_requests(&rf, cli.requests.as_deref())?;

    if cli.list_requests {
        for (name, _) in &requests {
            println!("\t{}", name);
        }
        return Ok(());
    }

    let output: Box<dyn OutputMode> = match cli.output {
        OutputFormat::Silent => Box::new(SilentOutput {}),
        OutputFormat::Quiet => Box::new(QuietOutput {}),
        OutputFormat::Verbose => Box::new(VerboseOutput {}),
        OutputFormat::ResponseOnly => Box::new(ResponseOnlyOutput {}),
        OutputFormat::Normal => Box::new(NormalOutput {}),
        OutputFormat::RequestOnly => Box::new(RequestOnlyOutput {}),
    };

    for (name, req) in &requests {
        let result = execute_request(name, req, &rf.vars, rf.config, output.as_ref());
        if let Err(e) = result {
            output.request_error(name, format!("{}", e).as_str());
            std::process::exit(1);
        }
    }

    Ok(())
}
