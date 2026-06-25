mod cli;
mod config;
mod git_snapshot;
mod index;
mod models;
mod render;
mod search;
mod store;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo = std::env::current_dir()?;

    match cli.command {
        Command::Init => store::init_repo(&repo)?,
        Command::SaveJson { feature } => {
            let memory = models::MemoryInput::from_reader(std::io::stdin())
                .context("failed to read memory JSON from stdin")?;
            store::save_memory(&repo, &feature, &memory)?;
        }
        Command::Load { feature, json } => {
            let context = store::load_context(&repo, &feature)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&context)?);
            } else {
                println!("{}", context.text);
            }
        }
        Command::List { json } => {
            let features = index::load_features(&repo)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&features)?);
            } else {
                for feature in features {
                    println!("{}", feature.name);
                }
            }
        }
        Command::Tree { json } => {
            let entries = store::memory_tree(&repo)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else {
                println!("{}", entries.join("\n"));
            }
        }
        Command::Search { query, json } => {
            let matches = search::keyword_search(&repo, &query)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&matches)?);
            } else {
                for item in matches {
                    println!("{}:{}: {}", item.path, item.line, item.snippet);
                }
            }
        }
        Command::Doctor { json } => {
            let report = store::doctor(&repo)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                for check in report.checks {
                    println!("{} {}", if check.ok { "ok" } else { "fail" }, check.message);
                }
            }
        }
        Command::Snapshot {
            feature,
            dry_run,
            json,
        } => {
            let snapshot = git_snapshot::collect(&repo, &feature, dry_run)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!("{}", render::render_snapshot(&snapshot));
            }
        }
    }

    Ok(())
}
