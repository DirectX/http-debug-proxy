use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub host_v6: Option<String>,
    pub port_v6: Option<u16>,
    pub api_key: Option<String>,
    pub ssl_cert: Option<String>,
    pub ssl_key: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub upstreams: HashMap<String, String>,
    pub default_upstream: Option<String>,
}