use clap::{Parser, Subcommand};
use clap_complete::Shell;
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
    /// Create the .promemo repository structure.
    Init,
    /// Start the Promemo MCP server over stdio.
    Mcp,
    /// Generate shell completion scripts.
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
    /// Generate Markdown CLI usage documentation.
    Docs,
    /// Manage the generated local search index.
    Index {
        #[command(subcommand)]
        command: IndexCommand,
    },
    /// Print machine-readable schemas.
    Schema {
        #[command(subcommand)]
        command: SchemaCommand,
    },
    /// Read or write memory through the stable JSON request envelope.
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    /// Extract structured memory from source text using the configured provider.
    Extract {
        /// Feature name to write extracted memory under.
        feature: String,
        /// Read extraction input from a file.
        #[arg(long, value_name = "FILE", conflicts_with = "stdin")]
        from: Option<PathBuf>,
        /// Read extraction input from stdin.
        #[arg(long, conflicts_with = "from")]
        stdin: bool,
        /// Preview writes without changing files.
        #[arg(long)]
        dry_run: bool,
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Save a MemoryInput JSON document for a feature.
    SaveJson {
        /// Feature name to save.
        feature: String,
        /// Preview writes without changing files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Parse and save assistant handoff Markdown for a feature.
    Save {
        /// Feature name to save.
        feature: String,
        /// Read handoff Markdown from a file.
        #[arg(long, value_name = "FILE", conflicts_with = "stdin")]
        from: Option<PathBuf>,
        /// Read handoff Markdown from stdin.
        #[arg(long, conflicts_with = "from")]
        stdin: bool,
        /// Print JSON output.
        #[arg(long)]
        json: bool,
        /// Preview writes without changing files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Load prompt-ready context for a feature.
    Load {
        /// Feature name to load.
        feature: String,
        /// Approximate maximum number of whitespace-delimited tokens to return.
        #[arg(long)]
        token_budget: Option<usize>,
        /// Include related feature summaries from the local relationship graph.
        #[arg(long)]
        related: bool,
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
    /// List saved feature names.
    List {
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Print saved memory files.
    Tree {
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Search saved memory.
    Search {
        /// Search query.
        query: String,
        /// Rank chunked results using local deterministic embeddings.
        #[arg(long, conflicts_with = "hybrid")]
        semantic: bool,
        /// Rank chunked results with keyword and semantic signals.
        #[arg(long, conflicts_with = "semantic")]
        hybrid: bool,
        /// Maximum number of matches to return.
        #[arg(long)]
        limit: Option<usize>,
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Validate the current Promemo repository.
    Doctor {
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Capture git state and TODO context for a feature.
    Snapshot {
        /// Feature name to snapshot.
        feature: String,
        /// Preview the snapshot without writing files.
        #[arg(long)]
        dry_run: bool,
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Open a feature directory in the OS file browser.
    Open {
        /// Feature name to open.
        feature: String,
    },
    /// Import memory from external formats.
    Import {
        #[command(subcommand)]
        command: ImportCommand,
    },
}

impl Cli {
    pub fn wants_json_errors(&self) -> bool {
        match &self.command {
            Command::Extract { json, .. }
            | Command::Save { json, .. }
            | Command::Load { json, .. }
            | Command::List { json, .. }
            | Command::Tree { json, .. }
            | Command::Search { json, .. }
            | Command::Doctor { json, .. }
            | Command::Snapshot { json, .. } => *json,
            Command::Import { command } => match command {
                ImportCommand::Handoff { json, .. } => *json,
            },
            Command::Index { command } => match command {
                IndexCommand::Rebuild { json } | IndexCommand::Status { json } => *json,
            },
            Command::Init
            | Command::Mcp
            | Command::Completions { .. }
            | Command::Docs
            | Command::Schema { .. }
            | Command::Memory { .. }
            | Command::SaveJson { .. }
            | Command::Open { .. } => false,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum IndexCommand {
    /// Rebuild the local search index.
    Rebuild {
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Show local search index status.
    Status {
        /// Print JSON output.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ImportCommand {
    /// Import assistant handoff Markdown from a file.
    Handoff {
        /// Handoff Markdown file to import.
        file: PathBuf,
        /// Feature name to save imported memory under.
        #[arg(long)]
        feature: String,
        /// Print JSON output.
        #[arg(long)]
        json: bool,
        /// Preview writes without changing files.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum SchemaCommand {
    /// Print the MemoryInput JSON schema.
    MemoryInput,
}

#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    /// Preview a memory save request without writing files.
    Preview,
    /// Save a memory save request.
    Save,
}
