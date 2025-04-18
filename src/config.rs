use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ClockConfig {
    pub time_format: String,
    pub date_format: String,
    pub time_font: String,
    pub date_font: String,
}
#[derive(Debug, Deserialize)]
pub struct WeatherConfig {
    pub font: String,
    pub location: String,
}
#[derive(Debug, Deserialize)]
pub struct Config {
    pub clock: ClockConfig,
    pub weather: WeatherConfig,
}

pub fn load_config(path: &str) -> Config {
    let toml_str = fs::read_to_string(path).expect("Failed to read config file");
    toml::from_str(&toml_str).expect("Failed to parse config file")
}
