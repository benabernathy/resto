use anyhow::{Result, anyhow};
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct RequestFile {
    #[serde(default)]
    pub config: Config,

    #[serde(default)]
    pub vars: HashMap<String, String>,

    #[serde(default)]
    pub profiles: HashMap<String, HashMap<String, String>>,

    #[serde(flatten)]
    pub requests: IndexMap<String, RequestDef>,
}

#[derive(Debug, Deserialize, Default, Clone, Copy)]
pub struct Config {
    #[serde(default)]
    pub ignore_ssl: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequestDef {
    #[serde(default = "default_method")]
    pub method: String,

    pub url: String,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default)]
    pub query: HashMap<String, String>,

    pub body: Option<String>,

    pub expect_status: Option<Vec<u16>>,

    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_method() -> String {
    "GET".to_string()
}
fn default_timeout() -> u64 {
    30
}

pub fn load_requests(rf: &RequestFile, name: Option<&str>) -> Result<Vec<(String, RequestDef)>> {
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
