use crate::cli::{Cli, Command};
use crate::{git_snapshot, handoff, index, models, render, search, store};
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::path::Path;

pub fn run(cli: Cli, cwd: &Path) -> Result<()> {
    run_with_io(cli, cwd, &mut std::io::stdin(), &mut std::io::stdout())
}

pub fn run_with_io(
    cli: Cli,
    cwd: &Path,
    stdin: &mut impl Read,
    stdout: &mut impl Write,
) -> Result<()> {
    match cli.command {
        Command::Init => store::init_repo(cwd)?,
        Command::SaveJson { feature } => {
            let repo = store::find_repo_root(cwd)?;
            let memory = models::MemoryInput::from_reader(stdin)
                .context("failed to read memory JSON from stdin")?;
            store::save_memory(&repo, &feature, &memory)?;
        }
        Command::Save {
            feature,
            from,
            stdin: use_stdin,
            json,
        } => {
            let repo = store::find_repo_root(cwd)?;
            let input = if use_stdin {
                let mut input = String::new();
                stdin
                    .read_to_string(&mut input)
                    .context("failed to read handoff Markdown from stdin")?;
                input
            } else if let Some(path) = from {
                std::fs::read_to_string(&path)
                    .with_context(|| format!("failed to read {}", path.display()))?
            } else {
                anyhow::bail!("provide either `--from <file>` or `--stdin`");
            };
            let memory = handoff::parse_markdown(&input)?;
            let report = store::save_memory_with_report(&repo, &feature, &memory)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&report)?)?;
            }
        }
        Command::Load { feature, json } => {
            let repo = store::find_repo_root(cwd)?;
            let context = store::load_context(&repo, &feature)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&context)?)?;
            } else {
                writeln!(stdout, "{}", context.text)?;
            }
        }
        Command::List { json } => {
            let repo = store::find_repo_root(cwd)?;
            let features = index::load_features(&repo)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&features)?)?;
            } else {
                for feature in features {
                    writeln!(stdout, "{}", feature.name)?;
                }
            }
        }
        Command::Tree { json } => {
            let repo = store::find_repo_root(cwd)?;
            let entries = store::memory_tree(&repo)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&entries)?)?;
            } else {
                writeln!(stdout, "{}", entries.join("\n"))?;
            }
        }
        Command::Search { query, json } => {
            let repo = store::find_repo_root(cwd)?;
            let matches = search::keyword_search(&repo, &query)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&matches)?)?;
            } else {
                for item in matches {
                    writeln!(stdout, "{}:{}: {}", item.path, item.line, item.snippet)?;
                }
            }
        }
        Command::Doctor { json } => {
            let repo = store::find_repo_root(cwd)?;
            let report = store::doctor(&repo)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                for check in report.checks {
                    writeln!(
                        stdout,
                        "{} {}",
                        if check.ok { "ok" } else { "fail" },
                        check.message
                    )?;
                }
            }
        }
        Command::Snapshot {
            feature,
            dry_run,
            json,
        } => {
            let repo = store::find_repo_root(cwd)?;
            let snapshot = git_snapshot::collect(&repo, &feature, dry_run)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&snapshot)?)?;
            } else {
                writeln!(stdout, "{}", render::render_snapshot(&snapshot))?;
            }
        }
    }

    Ok(())
}
