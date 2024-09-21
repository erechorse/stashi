extern crate serde;
extern crate toml;
extern crate reqwest;

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

#[derive(Debug, PartialEq)]
enum Status {
    Maintenance,
    Preopen,
    Open,
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
    fn get_status(&self) -> Status {
       let body = reqwest::blocking::get(
            format!("{}/public/v1/status", self.endpoint)).unwrap().text().unwrap();

        #[derive(Deserialize)]
        struct JSONData {
            pub status: String,
        }
        #[derive(Deserialize)]
        struct JSONResponse {
            status: i32,
            data: JSONData,
            responsetime: String,
        }

        let json: JSONResponse = serde_json::from_str(&body).unwrap();
        match &*(json.data.status) {
            "OPEN" => Status::Open,
            "PREOPEN" => Status::Preopen,
            _ => Status::Maintenance,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{APIcaller, Config, Status};

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

    #[test]
    fn test_get_session() {
        let config = Config::new("config_example.toml".to_string()).unwrap();
        let api_caller = APIcaller::new(config);
        assert_eq!(Status::Open, api_caller.get_status());
    }
    
}