use crate::redis::Configuration;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliArgs {
    /// The path to the directory where the RDB file is stored
    #[arg(long)]
    pub dir: Option<String>,
    /// The name of the RDB file
    #[arg(long)]
    pub dbfilename: Option<String>,
    /// The server port
    #[arg(long)]
    pub port: Option<u16>,
}

impl From<CliArgs> for Configuration {
    fn from(value: CliArgs) -> Self {
        Configuration::new(value.dir, value.dbfilename, value.port.unwrap_or(6379))
    }
}
