use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ClockConfig {
    pub time_format: String,
    pub date_format: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub clock: ClockConfig,
}

pub fn load_config(path: &str) -> Config {
    let toml_str = fs::read_to_string(path).expect("Failed to read config file");
    toml::from_str(&toml_str).expect("Failed to parse config file")
}
