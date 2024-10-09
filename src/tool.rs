use crate::{api::{PrivateAPICaller, PublicAPICaller, Status}, config::Config};

pub struct Tool {
    public: PublicAPICaller,
    private: PrivateAPICaller,
}

impl Tool {
    pub fn new(config: &Config, root_url: &str) -> Self {
        Self {
            public: PublicAPICaller::new(root_url.to_string()),
            private: PrivateAPICaller::new(config, root_url.to_string()),
        }
    }
    pub fn check(&self, config: &Config) -> Result<f64, Box<dyn std::error::Error>> {
        let status = self.public.get_status()?;
        match status {
            Status::Open => (),
            Status::Preopen => return Err(format!("Service is in pre-open.").into()),
            Status::Maintenance => return Err(format!("Service is in maintenance").into()),
        };
        
        let capacity = self.private.get_capacity()?;
        let price = self.public.get_price()?;

        let btc = ((&config.amount * 10_u32.pow(4) / price) as f64) / 10f64.powf(4.0);
        if btc == 0.0 {
            return Err(format!("The investment amount is less than the minimum transaction unit.").into());
        } 
        if capacity < btc as u32 {
            return Err(format!("Not enough margin.").into());
        }
        Ok(btc)
    }

    pub fn run(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let btc = self.check(&config)?;
        self.private.buy(btc)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::create_mock;

    use super::*;

    #[test]
    fn test_check() -> Result<(), Box<dyn std::error::Error>> {
        let server = mockito::Server::new();
        let status = (
            "/public/v1/status",
            r#"{"status":0,"data":{"status":"OPEN"},"responsetime":"2019-03-19T02:15:06.001Z"}"#
        );
        let capacity = (
            "/private/v1/account/margin",
            r#"{"status":0,"data":{"actualProfitLoss":"68286188","availableAmount":"57262506","margin":"1021682","marginCallStatus":"NORMAL","marginRatio":"6683.6","profitLoss":"0","transferableAmount":"57262506"},"responsetime":"2019-03-19T02:15:06.051Z"}"#
        );
        let price = (
            "/public/v1/ticker?symbol=BTC",
            r#"{"status":0,"data":[{"ask":"9343889","bid":"9343880","high":"9343889","last":"9343889","low":"9343800","symbol":"BTC","timestamp":"2018-03-30T12:34:56.789Z","volume":"194785.8484"}],"responsetime":"2019-03-19T02:15:06.014Z"}"#
        );
        let server = create_mock(server, "GET".to_string(), status.0.to_string(), status.1.to_string());
        let server = create_mock(server, "GET".to_string(), capacity.0.to_string(), capacity.1.to_string());
        let server = create_mock(server, "GET".to_string(), price.0.to_string(), price.1.to_string());

        let config = Config::new("config_example.toml")?;
        let tool = Tool::new(&config, &server.url());
        match tool.check(&config) {
            Ok(btc) => if btc == 0.0001 {
                Ok(())
            } else {
                Err(format!("The investment amount should be 0.0001.").into())
            }
            Err(error) => Err(error),
        }
    }

    #[test]
    fn test_invalid_check() -> Result<(), String> {
        let server = mockito::Server::new();
        let status = (
            "/public/v1/status",
            r#"{"status":0,"data":{"status":"OPEN"},"responsetime":"2019-03-19T02:15:06.001Z"}"#
        );
        let capacity = (
            "/private/v1/account/margin",
            r#"{"status":0,"data":{"actualProfitLoss":"68286188","availableAmount":"57262506","margin":"1021682","marginCallStatus":"NORMAL","marginRatio":"6683.6","profitLoss":"0","transferableAmount":"57262506"},"responsetime":"2019-03-19T02:15:06.051Z"}"#
        );
        let price = (
            "/public/v1/ticker?symbol=BTC",
            r#"{"status":0,"data":[{"ask":"9343889","bid":"9343880","high":"9343889","last":"9343889","low":"9343800","symbol":"BTC","timestamp":"2018-03-30T12:34:56.789Z","volume":"194785.8484"}],"responsetime":"2019-03-19T02:15:06.014Z"}"#
        );
        let server = create_mock(server, "GET".to_string(), status.0.to_string(), status.1.to_string());
        let server = create_mock(server, "GET".to_string(), capacity.0.to_string(), capacity.1.to_string());
        let server = create_mock(server, "GET".to_string(), price.0.to_string(), price.1.to_string());
        
        let config = Config {
            key: "my_api_key".to_string(),
            secret: "my_secret_key".to_string(),
            amount: 500
        };

        let tool = Tool::new(&config, &server.url());
        match tool.check(&config) {
            Ok(_) => Err("An error should be thrown when the amount is below the minimum quantity.".to_string()),
            Err(_) => Ok(()),    
        } 
    } 

    #[test]
    fn run_check() -> Result<(), Box<dyn std::error::Error>> {
        let server = mockito::Server::new();
        let status = (
            "/public/v1/status",
            r#"{"status":0,"data":{"status":"OPEN"},"responsetime":"2019-03-19T02:15:06.001Z"}"#
        );
        let capacity = (
            "/private/v1/account/margin",
            r#"{"status":0,"data":{"actualProfitLoss":"68286188","availableAmount":"57262506","margin":"1021682","marginCallStatus":"NORMAL","marginRatio":"6683.6","profitLoss":"0","transferableAmount":"57262506"},"responsetime":"2019-03-19T02:15:06.051Z"}"#
        );
        let price = (
            "/public/v1/ticker?symbol=BTC",
            r#"{"status":0,"data":[{"ask":"9343889","bid":"9343880","high":"9343889","last":"9343889","low":"9343800","symbol":"BTC","timestamp":"2018-03-30T12:34:56.789Z","volume":"194785.8484"}],"responsetime":"2019-03-19T02:15:06.014Z"}"#
        );
        let order = (
            "/private/v1/order",
            r#"{"status":0,"data":"637000","responsetime":"2019-03-19T02:15:06.108Z"}"#,
        );
        let server = create_mock(server, "GET".to_string(), status.0.to_string(), status.1.to_string());
        let server = create_mock(server, "GET".to_string(), capacity.0.to_string(), capacity.1.to_string());
        let server = create_mock(server, "GET".to_string(), price.0.to_string(), price.1.to_string());
        let server = create_mock(server, "POST".to_string(), order.0.to_string(), order.1.to_string());

        let config = Config::new("config_example.toml")?;
        let tool = Tool::new(&config, &server.url());
        tool.run(&config)?;
        Ok(())
    }
}