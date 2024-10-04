use std::time::{SystemTime, UNIX_EPOCH};

use ring::hmac;
use serde::{Deserialize, Serialize};

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
enum Status {
    Maintenance,
    Preopen,
    Open,
}

impl PublicAPICaller {
    fn new(root_url: String) -> Self {
        Self {
            endpoint: root_url + "/public",
        }
    }
    fn get_status(&self) -> Status {
       let body = reqwest::blocking::get(
            format!("{}/v1/status", self.endpoint)).unwrap().text().unwrap();

        #[derive(Serialize, Deserialize)]
        struct JSONData {
            pub status: String,
        }
        #[derive(Serialize, Deserialize)]
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

impl PrivateAPICaller {
    fn new(config: Config, root_url: String) -> Self {
        Self {
            endpoint: root_url + "/private",
            key: config.key,
            secret: config.secret,
        }
    }
    fn get_capacity(&self) -> u32 {
        let path = "/v1/account/margin";
        let time = PrivateAPICaller::get_timestamp();
        let sign = self.sign(time, "GET".to_string(), path.to_string());
        let client = reqwest::blocking::Client::new();
        let res = client.get(self.endpoint.to_string() + path)
            .header("API-KEY", &self.key)
            .header("API-TIMESTAMP", time)
            .header("API-SIGN", sign)
            .send()
            .unwrap()
            .text()
            .unwrap();
        #[derive(Serialize, Deserialize)]
        struct JSONResponse {
            status: i32,
            data: JSONData,
            responsetime: String,
        }
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

        let json: JSONResponse = serde_json::from_str(&res).unwrap();
        json.data.availableAmount.parse().unwrap()
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
    use super::*;

    #[test]
    fn test_get_session() {
        let mut server = mockito::Server::new();
        let path = "/public/v1/status";
        let body = r#"{"status":0,"data":{"status":"OPEN"},"responsetime":"2019-03-19T02:15:06.001Z"}"#;
        let _mock = server.mock("GET", path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let api_caller = PublicAPICaller::new(server.url()); 
        assert_eq!(Status::Open, api_caller.get_status());
    }

    #[test]
    fn test_get_capacity() {
        let mut server = mockito::Server::new();
        let path = "/private/v1/account/margin";
        let body = r#"{"status":0,"data":{"actualProfitLoss":"68286188","availableAmount":"57262506","margin":"1021682","marginCallStatus":"NORMAL","marginRatio":"6683.6","profitLoss":"0","transferableAmount":"57262506"},"responsetime":"2019-03-19T02:15:06.051Z"}"#;
        let _mock = server.mock("GET", path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let config = Config::new("config.toml".to_string()).unwrap();
        let api_caller = PrivateAPICaller::new(config, server.url());
        assert_eq!(api_caller.get_capacity(), 57262506);
    }

    #[test]
    fn test_sign() {
        let config = Config::new("config_example.toml".to_string()).unwrap();
        let api_caller = PrivateAPICaller::new(config, "".to_string());

        let time = 1727601179;
        let path = "/v1/account/margin";
        let method = "GET";
        let signature = "e8113c9454190c7cc8e3860012bae623bc36f2061b99660577b7c0bf22ea3f62";

        assert_eq!(api_caller.sign(time, method.to_string(), path.to_string()), signature);
    }
}