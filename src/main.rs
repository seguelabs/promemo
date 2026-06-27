use anyhow::Result;
use clap::Parser;
use promemo::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;
    promemo::app::run(cli, &cwd)
}
