use anyhow::Result;
use clap::Parser;
use promem::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;
    promem::app::run(cli, &cwd)
}
