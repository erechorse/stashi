use crate::{api::{PrivateAPICaller, PublicAPICaller, Status}, config::Config};

fn check(config_path: String, root_url: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let config = Config::new(config_path)?;
    let public = PublicAPICaller::new(root_url.to_string());
    let private = PrivateAPICaller::new(&config, root_url.to_string());

    let status = public.get_status()?;
    match status {
        Status::Open => (),
        Status::Preopen => return Err(format!("Service is in pre-open.").into()),
        Status::Maintenance => return Err(format!("Service is in maintenance").into()),
    };
    
    let capacity = private.get_capacity()?;
    let price = public.get_price()?;

    let btc = ((&config.amount * 10_u32.pow(4) / price) as f64) / 10f64.powf(4.0);
    if btc == 0.0 {
        return Err(format!("The investment amount is less than the minimum transaction unit.").into());
    } 
    if capacity < btc as u32 {
        return Err(format!("Not enough margin.").into());
    }
    Ok(btc as u32)
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
            r#"{"status":0,"data":[{"ask":"750760","bid":"750600","high":"762302","last":"756662","low":"704874","symbol":"BTC","timestamp":"2018-03-30T12:34:56.789Z","volume":"194785.8484"}],"responsetime":"2019-03-19T02:15:06.014Z"}"#
        );
        let server = create_mock(server, "GET".to_string(), status.0.to_string(), status.1.to_string());
        let server = create_mock(server, "GET".to_string(), capacity.0.to_string(), capacity.1.to_string());
        let server = create_mock(server, "GET".to_string(), price.0.to_string(), price.1.to_string());

        match check("config_example.toml".to_string(), &server.url()) {
            Ok(btc) => Ok(()),
            Err(error) => Err(error),
        }
    }
}