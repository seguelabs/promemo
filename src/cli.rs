use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "promemo")]
#[command(about = "Git-native project memory for AI-assisted development")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    Schema {
        #[command(subcommand)]
        command: SchemaCommand,
    },
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    SaveJson {
        feature: String,
        #[arg(long)]
        dry_run: bool,
    },
    Save {
        feature: String,
        #[arg(long, value_name = "FILE", conflicts_with = "stdin")]
        from: Option<PathBuf>,
        #[arg(long, conflicts_with = "from")]
        stdin: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        dry_run: bool,
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
    Tree {
        #[arg(long)]
        json: bool,
    },
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
    Open {
        feature: String,
    },
    Import {
        #[command(subcommand)]
        command: ImportCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ImportCommand {
    Handoff {
        file: PathBuf,
        #[arg(long)]
        feature: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum SchemaCommand {
    MemoryInput,
}

#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    Preview,
    Save,
}
