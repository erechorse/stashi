use std::env;

use seahorse::{App, Command, Context};

use stashi::{config::Config, tool::Tool};

const ROOT_URL: &str = "https://api.coin.z.com";

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("stashi [command]")
        .command(check_command())
        .command(run_command());
    app.run(args);
}

fn check_command() -> Command {
    Command::new("check")
        .description("Check if the investment is possible")
        .alias("c")
        .usage("stashi check(c) [Config.toml]")
        .action(check)
}

fn run_command() -> Command {
    Command::new("run")
        .description("Execute the investment.")
        .alias("r")
        .usage("stashi run(r) [Config.toml]")
        .action(run)
}

fn check(c: &Context) {
    let args = &c.args;
    if args.len() == 0 {
        eprintln!("Please specify the path to the configuration file.");
        return;
    }
    let config = match Config::new(&args[0]) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{}", error.to_string());
            return;
        }
    };
    let tool = Tool::new(&config, ROOT_URL);
    let btc = match tool.check() {
        Ok(btc) => btc,
        Err(error) => {
            eprintln!("{}", error.to_string());
            return;
        }
    };
    println!("You can accumulate {} BTC.", btc);
}

fn run(c: &Context) {
    let args = &c.args;
    if args.len() == 0 {
        eprintln!("Please specify the path to the configuration file.");
        return;
    }
    let config = match Config::new(&args[0]) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{}", error.to_string());
            return;
        }
    };
    let tool = Tool::new(&config, ROOT_URL);
    let btc = match tool.run() {
        Ok(btc) => btc,
        Err(error) => {
            eprintln!("{}", error.to_string());
            return;
        }
    };
    println!("The accumulation of {} BTC has been executed.", btc);
}