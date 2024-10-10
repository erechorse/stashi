use crate::{api::{PrivateAPICaller, PublicAPICaller, Status}, config::Config};

pub struct Tool {
    public: PublicAPICaller,
    private: PrivateAPICaller,
    amount: u32,
}

impl Tool {
    pub fn new(config: &Config, root_url: &str) -> Self {
        Self {
            public: PublicAPICaller::new(root_url),
            private: PrivateAPICaller::new(config, root_url),
            amount: config.amount,
        }
    }
    pub fn check(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let status = self.public.get_status()?;
        match status {
            Status::Open => (),
            Status::Preopen => return Err(format!("Service is in pre-open.").into()),
            Status::Maintenance => return Err(format!("Service is in maintenance").into()),
        };
        
        let capacity = self.private.get_capacity()?;
        let price = self.public.get_price()?;

        let btc = ((self.amount * 10_u32.pow(4) / price) as f64) / 10f64.powf(4.0);
        if btc == 0.0 {
            return Err(format!("The investment amount is less than the minimum transaction unit.").into());
        } 
        if capacity < btc as u32 {
            return Err(format!("Not enough margin.").into());
        }
        Ok(btc)
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let btc = self.check()?;
        self.private.buy(btc)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::TestServer;

    use super::*;

    #[test]
    fn test_check() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = TestServer::new(mockito::Server::new());
        server.create_mock(
            "GET", 
            "/public/v1/status",
            r#"{"status":0,"data":{"status":"OPEN"},"responsetime":"2019-03-19T02:15:06.001Z"}"#
        );
        server.create_mock(
            "GET", 
            "/private/v1/account/margin",
            r#"{"status":0,"data":{"actualProfitLoss":"68286188","availableAmount":"57262506","margin":"1021682","marginCallStatus":"NORMAL","marginRatio":"6683.6","profitLoss":"0","transferableAmount":"57262506"},"responsetime":"2019-03-19T02:15:06.051Z"}"#
        );
        server.create_mock(
            "GET", 
            "/public/v1/ticker?symbol=BTC",
            r#"{"status":0,"data":[{"ask":"9343889","bid":"9343880","high":"9343889","last":"9343889","low":"9343800","symbol":"BTC","timestamp":"2018-03-30T12:34:56.789Z","volume":"194785.8484"}],"responsetime":"2019-03-19T02:15:06.014Z"}"#
        );

        let config = Config::new("config_example.toml")?;
        let tool = Tool::new(&config, &server.url());
        match tool.check() {
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
        let mut server = TestServer::new(mockito::Server::new());
        server.create_mock(
            "GET", 
            "/public/v1/status",
            r#"{"status":0,"data":{"status":"OPEN"},"responsetime":"2019-03-19T02:15:06.001Z"}"#
        );
        server.create_mock(
            "GET", 
            "/private/v1/account/margin",
            r#"{"status":0,"data":{"actualProfitLoss":"68286188","availableAmount":"57262506","margin":"1021682","marginCallStatus":"NORMAL","marginRatio":"6683.6","profitLoss":"0","transferableAmount":"57262506"},"responsetime":"2019-03-19T02:15:06.051Z"}"#
        );
        server.create_mock(
            "GET", 
            "/public/v1/ticker?symbol=BTC",
            r#"{"status":0,"data":[{"ask":"9343889","bid":"9343880","high":"9343889","last":"9343889","low":"9343800","symbol":"BTC","timestamp":"2018-03-30T12:34:56.789Z","volume":"194785.8484"}],"responsetime":"2019-03-19T02:15:06.014Z"}"#
        );
        
        let config = Config {
            key: "my_api_key".to_string(),
            secret: "my_secret_key".to_string(),
            amount: 500
        };

        let tool = Tool::new(&config, &server.url());
        match tool.check() {
            Ok(_) => Err("An error should be thrown when the amount is below the minimum quantity.".to_string()),
            Err(_) => Ok(()),    
        } 
    } 

    #[test]
    fn run_check() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = TestServer::new(mockito::Server::new());
        server.create_mock(
            "GET", 
            "/public/v1/status",
            r#"{"status":0,"data":{"status":"OPEN"},"responsetime":"2019-03-19T02:15:06.001Z"}"#
        );
        server.create_mock(
            "GET", 
            "/private/v1/account/margin",
            r#"{"status":0,"data":{"actualProfitLoss":"68286188","availableAmount":"57262506","margin":"1021682","marginCallStatus":"NORMAL","marginRatio":"6683.6","profitLoss":"0","transferableAmount":"57262506"},"responsetime":"2019-03-19T02:15:06.051Z"}"#
        );
        server.create_mock(
            "GET", 
            "/public/v1/ticker?symbol=BTC",
            r#"{"status":0,"data":[{"ask":"9343889","bid":"9343880","high":"9343889","last":"9343889","low":"9343800","symbol":"BTC","timestamp":"2018-03-30T12:34:56.789Z","volume":"194785.8484"}],"responsetime":"2019-03-19T02:15:06.014Z"}"#
        );
        server.create_mock(
            "POST", 
            "/private/v1/order",
            r#"{"status":0,"data":"637000","responsetime":"2019-03-19T02:15:06.108Z"}"#
        );

        let config = Config::new("config_example.toml")?;
        let tool = Tool::new(&config, &server.url());
        tool.run()?;
        Ok(())
    }
}