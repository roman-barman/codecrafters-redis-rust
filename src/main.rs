use crate::cli_args::CliArgs;
use crate::config::Config;
use crate::redis::Server;
use clap::Parser;
use simple_logger::SimpleLogger;

mod cli_args;
mod config;
mod redis;

fn main() {
    SimpleLogger::new().init().unwrap();

    let args = CliArgs::parse();
    let mut redis = Server::new(Config::from(args));
    redis.run();
}
