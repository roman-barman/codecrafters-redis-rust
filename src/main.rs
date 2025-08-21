use crate::cli_args::CliArgs;
use crate::config::Config;
use crate::redis::Redis;
use clap::Parser;

mod cli_args;
mod command_parsers;
mod commands;
mod config;
mod engine;
mod handlers;
mod mediators;
mod redis;
mod resp;
mod storages;

fn main() {
    println!("Logs from your program will appear here!");

    let args = CliArgs::parse();
    let redis = Redis::new(Config::from(args));
    redis.run();
}
