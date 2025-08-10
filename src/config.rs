use crate::cli_args::CliArgs;

pub struct Config {
    pub dir: Option<String>,
    pub dbfilename: Option<String>,
}

impl From<CliArgs> for Config {
    fn from(value: CliArgs) -> Self {
        Self {
            dir: value.dir,
            dbfilename: value.dbfilename,
        }
    }
}