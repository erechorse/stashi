extern crate serde;
extern crate toml;

use std::{fs::File, io::Read};
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub key: String,
    pub secret: String,
    pub amount: i32,
}

pub struct APIcaller {
    endpoint: String,
    key: String,
    secret: String,
}


fn main() {
    println!("Hello, world!");
}

impl Config {
    fn new(path: String) -> Result<Self, String> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(e.to_string()),
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Err(e) => return Err(e.to_string()),
            _ => (),
        };
        let config: Result<Config, toml::de::Error> = toml::from_str(&contents);
        match config {
            Ok(config) => return Ok(config),
            Err(e) => return Err(e.to_string()),
        };
    }
}

impl APIcaller {
    fn new(config: Config) -> Self {
        Self {
            endpoint: "https://api.coin.z.com".to_string(),
            key: config.key,
            secret: config.secret,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;

    #[test]
    fn test_config() {
        let config_path = "config_example.toml";
        let config = Config::new(config_path.to_string());
        let test_case = Config {
            key: "your_api_key".to_string(),
            secret: "your_secret_key".to_string(),
            amount: 1000,
        };
        assert_eq!(config, Ok(test_case));
    }

    
}