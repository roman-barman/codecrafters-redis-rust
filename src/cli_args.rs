use crate::redis::Configuration;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliArgs {
    /// The path to the directory where the RDB file is stored
    #[arg(long)]
    dir: Option<String>,
    /// The name of the RDB file
    #[arg(long)]
    dbfilename: Option<String>,
    /// The server port
    #[arg(long)]
    port: Option<u16>,
    // Replication info
    #[arg(long)]
    replicaof: Option<String>,
}

impl From<CliArgs> for Configuration {
    fn from(value: CliArgs) -> Self {
        Configuration::new(
            value.dir,
            value.dbfilename,
            value.port.unwrap_or(6379),
            value.replicaof,
        )
    }
}
