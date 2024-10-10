use std::{time::{SystemTime, UNIX_EPOCH}, u32};

use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::Config;

pub struct PublicAPICaller {
    endpoint: String,
}

pub struct PrivateAPICaller {
    endpoint: String,
    key: String,
    secret: String, 
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Maintenance,
    Preopen,
    Open,
}

#[derive(Serialize, Deserialize)]
struct JSONResponse<T> {
    status: i32,
    data: T,
    responsetime: String,
}

impl PublicAPICaller {
    pub fn new(root_url: &str) -> Self {
        Self {
            endpoint: format!("{}/public", root_url),
        }
    }
    pub fn get_status(&self) -> Result<Status, Box<dyn std::error::Error>> {
       let body = reqwest::blocking::get(
            format!("{}/v1/status", self.endpoint))?.text()?;

        #[derive(Serialize, Deserialize)]
        struct JSONData {
            pub status: String,
        }

        let json: JSONResponse<JSONData> = serde_json::from_str(&body)?;
        Ok(match &*(json.data.status) {
            "OPEN" => Status::Open,
            "PREOPEN" => Status::Preopen,
            _ => Status::Maintenance,
        })
    }
    pub fn get_price(&self) -> Result<u32, Box<dyn std::error::Error>> {
       let body = reqwest::blocking::get(
            format!("{}/v1/ticker?symbol=BTC", self.endpoint))?.text()?;
        
        #[derive(Serialize, Deserialize)]
        struct JSONData {
            ask: String,
            bid: String,
            high: String,
            last: String,
            low: String,
            symbol: String,
            timestamp: String,
            volume: String,
        }

        let json: JSONResponse<Vec<JSONData>> = serde_json::from_str(&body)?;
        Ok(json.data[0].ask.parse()?)
    }
}

impl PrivateAPICaller {
    pub fn new(config: &Config, root_url: &str) -> Self {
        Self {
            endpoint: format!("{}/private", root_url),
            key: config.key.to_string(),
            secret: config.secret.to_string(),
        }
    }
    pub fn get_capacity(&self) -> Result<u32, Box<dyn std::error::Error>> { 
        let path = "/v1/account/margin";
        let time = PrivateAPICaller::get_timestamp();
        let sign = self.sign(time, "GET".to_string(), path.to_string());
        let client = reqwest::blocking::Client::new();
        let res = client.get(self.endpoint.to_string() + path)
            .header("API-KEY", &self.key)
            .header("API-TIMESTAMP", time)
            .header("API-SIGN", sign)
            .send()?
            .text()?;

        #[derive(Serialize, Deserialize)]
        #[allow(non_snake_case)]
        struct JSONData {
            actualProfitLoss: String,
            availableAmount: String,
            margin: String,
            marginCallStatus: String,
            profitLoss: String,
            transferableAmount: String,
        }

        let json: JSONResponse<JSONData> = serde_json::from_str(&res)?;
        Ok(json.data.availableAmount.parse()?)
    }
    pub fn buy(&self, size: f64) -> Result<(), Box<dyn std::error::Error>> {
        let path = "/v1/order";
        let time = PrivateAPICaller::get_timestamp();
        let parameters = json!({
            "symbol": "BTC_JPY",
            "side": "BUY",
            "executionType": "MARKET",
            "size": size.to_string(),
        });
        let sign = self.sign(time, "POST".to_string(), path.to_string());
        let client = reqwest::blocking::Client::new();
        let res = client.post(self.endpoint.to_string() + path)
            .header("API-KEY", &self.key)
            .header("API-TIMESTAMP", time)
            .header("API-SIGN", sign)
            .json(&parameters)
            .send()?
            .text()?;

        let json: JSONResponse<String> = serde_json::from_str(&res)?;
        match json.status {
            0 => Ok(()),
            _ => Err(format!("Error: Status code {}", json.status).into()),
        }
    }

    fn sign(&self, time: u64, method: String, path: String) -> String {
        let text = format!("{}{}{}", time, method, path);
        let signed_key = hmac::Key::new(hmac::HMAC_SHA256, self.secret.as_bytes());
        hex::encode(hmac::sign(&signed_key, text.as_bytes()).as_ref())
    }

    fn get_timestamp() -> u64 {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards", );
    
        since_epoch.as_secs() * 1000 + since_epoch.subsec_nanos() as u64 / 1_000_000
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::TestServer;

    use super::*;

    #[test]
    fn test_get_status() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = TestServer::new(mockito::Server::new());
        server.create_mock(
            "GET", 
            "/public/v1/status",
            r#"{"status":0,"data":{"status":"OPEN"},"responsetime":"2019-03-19T02:15:06.001Z"}"# 
        );

        let api_caller = PublicAPICaller::new(&server.url()); 
        match api_caller.get_status() {
            Ok(status) => match status {
                Status::Open => Ok(()),
                _ => Err(format!("The function returns {:?}", status).into()),
            },
            Err(error) => Err(error),
        }
    }

    #[test]
    fn test_get_capacity() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = TestServer::new(mockito::Server::new());
        server.create_mock(
            "GET", 
            "/private/v1/account/margin",
            r#"{"status":0,"data":{"actualProfitLoss":"68286188","availableAmount":"57262506","margin":"1021682","marginCallStatus":"NORMAL","marginRatio":"6683.6","profitLoss":"0","transferableAmount":"57262506"},"responsetime":"2019-03-19T02:15:06.051Z"}"# 
        );

        let config = Config::new("config.toml")?;
        let api_caller = PrivateAPICaller::new(&config, &server.url());
        match api_caller.get_capacity() {
            Ok(capacity) => match capacity {
                57262506 => Ok(()),
                _ => Err(format!("The function returns {}", capacity).into()),
            },
            Err(error) => Err(error),
        }
    }

    #[test]
    fn test_get_price() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = TestServer::new(mockito::Server::new());
        server.create_mock(
            "GET", 
            "/public/v1/ticker?symbol=BTC",
            r#"{"status":0,"data":[{"ask":"750760","bid":"750600","high":"762302","last":"756662","low":"704874","symbol":"BTC","timestamp":"2018-03-30T12:34:56.789Z","volume":"194785.8484"}],"responsetime":"2019-03-19T02:15:06.014Z"}"# 
        );

        let api_caller = PublicAPICaller::new(&server.url());
        match api_caller.get_price() {
            Ok(price) => match price {
                750760 => Ok(()),
                _ => Err(format!("The function returns {}", price).into()),
            },
            Err(error) => Err(error),
        }
    }

    #[test]
    fn test_buy() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = TestServer::new(mockito::Server::new());
        server.create_mock(
            "POST", 
            "/private/v1/order",
            r#"{"status":0,"data":"637000","responsetime":"2019-03-19T02:15:06.108Z"}"# 
        );

        let config = Config::new("config.toml").unwrap();
        let api_caller = PrivateAPICaller::new(&config, &server.url());
        api_caller.buy(0.0001)
    }
    #[test]
    fn test_sign() {
        let config = Config::new("config_example.toml").unwrap();
        let api_caller = PrivateAPICaller::new(&config, "");

        let time = 1727601179;
        let path = "/v1/account/margin";
        let method = "GET";
        let signature = "e8113c9454190c7cc8e3860012bae623bc36f2061b99660577b7c0bf22ea3f62";

        assert_eq!(api_caller.sign(time, method.to_string(), path.to_string()), signature);
    }


}