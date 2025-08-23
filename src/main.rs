use crate::cli_args::CliArgs;
use crate::config::Config;
use crate::redis::Server;
use clap::Parser;

mod cli_args;
mod command_parsers;
mod commands;
mod config;
mod core;
mod handlers;
mod redis;
mod resp;
mod storages;

fn main() {
    println!("Logs from your program will appear here!");

    let args = CliArgs::parse();
    let redis = Server::new(Config::from(args));
    redis.run();
}
