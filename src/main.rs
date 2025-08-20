use crate::cli_args::CliArgs;
use crate::config::Config;
use crate::redis::Redis;
use clap::Parser;

mod commands;
mod resp;
mod storages;
mod cli_args;
mod config;
mod handlers;
mod mediators;
mod command_parsers;
mod engine;
mod redis;

fn main() {
    println!("Logs from your program will appear here!");

    let args = CliArgs::parse();
    let redis = Redis::new(Config::from(args));
    redis.run();
}
