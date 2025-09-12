use crate::cli_args::CliArgs;
use crate::redis::{Configuration, Server};
use clap::Parser;
use simple_logger::SimpleLogger;

mod cli_args;
mod redis;

fn main() {
    SimpleLogger::new().init().unwrap();

    let args = CliArgs::parse();
    let mut redis = Server::new(Configuration::from(args));
    redis.run();
}
