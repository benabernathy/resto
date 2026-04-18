use anyhow::{Context, Result, anyhow};
use clap::Parser;
use colored::*;
use indexmap::IndexMap;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "resto", about = "A developer-friendly REST client")]
struct Cli {
    /// Path to the collection file
    file: PathBuf,

    /// Name of the request to run (omits runs all)
    requests: Option<String>,

    // Only print the request name and the HTTP status code
    #[arg(short, long)]
    quiet: bool,

    // Exit 0 on success, 1 on failure - no output
    #[arg(short = 'Q', long)]
    silent: bool,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Deserialize)]
struct RequestFile {
    #[serde(default)]
    vars: HashMap<String, String>,
    #[serde(flatten)]
    requests: IndexMap<String, RequestDef>,
}

#[derive(Debug, Deserialize, Clone)]
struct RequestDef {
    #[serde(default = "default_method")]
    method: String,
    url: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    query: HashMap<String, String>,
    body: Option<String>,
    content_type: Option<String>,
    expect_status: Option<Vec<u16>>,
    #[serde(default = "default_timeout")]
    timeout_secs: u64,
}

fn default_method() -> String {
    "GET".to_string()
}
fn default_timeout() -> u64 {
    30
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let content = fs::read_to_string(&cli.file)
        .with_context(|| format!("could not read {}", cli.file.display()))?;

    let rf: RequestFile = toml::from_str(&content)
        .with_context(|| format!("could not parse {}", cli.file.display()))?;

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

fn interpolate(s: &str, vars: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (k, v) in vars {
        result = result.replace(&format!("{{{{{k}}}}}"), v);
    }

    result
}

fn load_requests(rf: &RequestFile, name: Option<&str>) -> Result<Vec<(String, RequestDef)>> {
    match name {
        Some(n) => {
            let req = rf.requests.get(n).cloned().ok_or_else(|| {
                let available: Vec<&String> = rf.requests.keys().collect();
                anyhow!("no request named '{}'. Available: {:?}", n, available)
            })?;
            Ok(vec![(n.to_string(), req)])
        }
        None => Ok(rf
            .requests
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()),
    }
}

fn execute_request(
    name: &str,
    req: &RequestDef,
    vars: &HashMap<String, String>,
    quiet: bool,
    silent: bool,
    verbose: bool,
) -> Result<()> {
    let url = interpolate(&req.url, vars);
    let method = req.method.to_uppercase();

    // build headers
    let mut header_map = HeaderMap::new();
    for (k, v) in &req.headers {
        let v = interpolate(v, vars);
        header_map.insert(HeaderName::from_str(k)?, HeaderValue::from_str(&v)?);
    }

    // build body
    let body_bytes = if let Some(body) = &req.body {
        let interpolated = interpolate(body, vars);

        // validate it's real JSON
        serde_json::from_str::<serde_json::Value>(&interpolated)
            .with_context(|| format!("body in '{name} is not valid JSON"))?;
        header_map.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        Some(interpolated.into_bytes())
    } else {
        None
    };

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(req.timeout_secs))
        .build()?;

    let mut builder = client
        .request(reqwest::Method::from_bytes(method.as_bytes())?, &url)
        .headers(header_map.clone());

    if !req.query.is_empty() {
        let query: Vec<(String, String)> = req
            .query
            .iter()
            .map(|(k, v)| (k.clone(), interpolate(v, vars)))
            .collect();

        let url = reqwest::Url::parse_with_params(&url, &query)
            .with_context(|| format!("could not build query params for '{name}'"))?;

        builder = client
            .request(reqwest::Method::from_bytes(method.as_bytes())?, url)
            .headers(header_map.clone());
    }

    if let Some(b) = body_bytes {
        builder = builder.body(b);
    }

    if verbose && !quiet && !silent {
        println!("{}", "-- request -------------------------------".dimmed());
        println!("{} {}", req.method.to_uppercase().cyan().bold(), url);
        if !req.query.is_empty() {
            println!("{}", "query:".dimmed());
            for (k, v) in &req.query {
                println!("  {} = {}", k.dimmed(), interpolate(v, vars));
            }
        }

        if !header_map.is_empty() {
            println!("{}", "headers".dimmed());
            for (k, v) in &header_map {
                println!(
                    "  {}: {}",
                    k.as_str().dimmed(),
                    v.to_str().unwrap_or("<binary>")
                );
            }
        }

        if let Some(body) = &req.body {
            println!("{}", "body".dimmed());
            println!("{}", try_pretty_json(&interpolate(body, vars)));
        }
    }

    let start = Instant::now();
    let response = builder
        .send()
        .with_context(|| format!("request '{name}' failed to send"))?;
    let elapsed = start.elapsed();

    let status = response.status();
    let body_text = response.text().unwrap_or_default();

    // print result
    let code = status.as_u16();
    let status_colored = if status.is_success() {
        code.to_string().green()
    } else if status.is_client_error() {
        code.to_string().yellow()
    } else if status.is_server_error() {
        code.to_string().red()
    } else {
        code.to_string().white()
    };

    if silent {
        // no output at all
    } else if quiet {
        println!("[{}] {} ({:.0?})", name, status_colored, elapsed);
    } else {
        println!("[{}] {} ({:.0?})", name, status_colored, elapsed);
        if verbose {
            println!("{}", "-- response -------------------------------".dimmed());
        }
        println!("{}", try_pretty_json(&body_text));
    }

    if let Some(expected) = &req.expect_status {
        let code = status.as_u16();
        if !expected.contains(&code) {
            return Err(anyhow!(
                "request '{}' expected status {:?} but got {}",
                name,
                expected,
                code
            ));
        }
    }

    Ok(())
}

fn try_pretty_json(s: &str) -> String {
    serde_json::from_str::<serde_json::Value>(s)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| s.to_string())
}
