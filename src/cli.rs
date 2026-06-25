use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "promem")]
#[command(about = "Git-native project memory for AI-assisted development")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    SaveJson {
        feature: String,
    },
    Load {
        feature: String,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Tree,
    Search {
        query: String,
        #[arg(long)]
        json: bool,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Snapshot {
        feature: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
}
