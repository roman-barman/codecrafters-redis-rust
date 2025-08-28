use crate::cli_args::CliArgs;
use crate::config::Config;
use crate::redis::Server;
use clap::Parser;

mod cli_args;
mod config;
mod redis;

fn main() {
    println!("Logs from your program will appear here!");

    let args = CliArgs::parse();
    let mut redis = Server::new(Config::from(args));
    redis.run();
}
