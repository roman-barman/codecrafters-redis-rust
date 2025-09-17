use crate::cli_args::CliArgs;
use crate::redis::{Configuration, Server};
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod cli_args;
mod redis;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let args = CliArgs::parse();
    let mut redis = Server::new(Configuration::from(args));
    redis.run();
}
