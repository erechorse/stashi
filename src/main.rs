use std::env;

use seahorse::{App, Command, Context};

use stashi::{config::Config, tool};

const ROOT_URL: &str = "https://api.coin.z.com";

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("stashi [name]")
        .command(check_command());
    app.run(args);
}

fn check_command() -> Command {
    Command::new("check")
        .description("Check if the investment is possible")
        .alias("c")
        .usage("stashi check(c) [CONFIG.toml]")
        .action(check)
}

fn check(c: &Context) {
    let config = match Config::new(&c.args[0]) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{}", error.to_string());
            return;
        }
    };
    let btc = match tool::check(&config, ROOT_URL) {
        Ok(btc) => btc,
        Err(error) => {
            eprintln!("{}", error.to_string());
            return;
        }
    };
    println!("You can accumulate {} BTC.", btc);
}