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
}

impl From<CliArgs> for Configuration {
    fn from(value: CliArgs) -> Self {
        Self {
            dir: value.dir,
            db_file_name: value.dbfilename,
        }
    }
}
