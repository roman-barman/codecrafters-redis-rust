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
