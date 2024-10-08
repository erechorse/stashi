use std::{fs::File, io::Read};

use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub key: String,
    pub secret: String,
    pub amount: u32,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>>  {
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() -> Result<(), Box<dyn std::error::Error>> {
        let config_path = "config_example.toml";
        let config = Config::new(config_path)?;
        let test_case = Config {
            key: "your_api_key".to_string(),
            secret: "your_secret_key".to_string(),
            amount: 1000,
        };
        if config == test_case {
            Ok(())
        } else {
            Err(format!("The Config returns {:?}", config).into())
        }
    }
}