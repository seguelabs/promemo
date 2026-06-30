use crate::cli::{Cli, Command, ImportCommand, IndexCommand, MemoryCommand, SchemaCommand};
use crate::{config, git_snapshot, handoff, index, mcp, models, provider, render, search, store};
use anyhow::{bail, Context, Result};
use clap::CommandFactory;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command as ProcessCommand;

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
        Command::Mcp => mcp::serve(cwd, stdin, stdout)?,
        Command::Completions { shell } => {
            let mut command = Cli::command();
            clap_complete::generate(shell, &mut command, "promemo", stdout);
        }
        Command::Docs => {
            write_cli_docs(stdout)?;
        }
        Command::Index { command } => {
            let repo = store::find_repo_root(cwd)?;
            match command {
                IndexCommand::Rebuild { json } => {
                    let built = search::rebuild_index(&repo)?;
                    if json {
                        writeln!(stdout, "{}", serde_json::to_string_pretty(&built)?)?;
                    } else {
                        writeln!(stdout, "Rebuilt search index")?;
                        writeln!(stdout, "Chunks: {}", built.chunks.len())?;
                        writeln!(stdout, "Features: {}", built.graph.features.len())?;
                        writeln!(stdout, "Files: {}", built.graph.files.len())?;
                        writeln!(stdout, "Decisions: {}", built.graph.decisions.len())?;
                    }
                }
                IndexCommand::Status { json } => {
                    let status = search::index_status(&repo)?;
                    if json {
                        writeln!(stdout, "{}", serde_json::to_string_pretty(&status)?)?;
                    } else if status.exists {
                        writeln!(stdout, "Search index: {}", status.path)?;
                        writeln!(stdout, "Version: {}", status.version.unwrap_or_default())?;
                        writeln!(stdout, "Chunks: {}", status.chunks)?;
                        writeln!(stdout, "Features: {}", status.features)?;
                        writeln!(stdout, "Files: {}", status.files)?;
                        writeln!(stdout, "Decisions: {}", status.decisions)?;
                    } else {
                        writeln!(stdout, "Search index does not exist yet")?;
                        writeln!(stdout, "Run: promemo index rebuild")?;
                    }
                }
            }
        }
        Command::Schema { command } => match command {
            SchemaCommand::MemoryInput => {
                writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string_pretty(&models::memory_input_schema())?
                )?;
            }
        },
        Command::Memory { command } => {
            let repo = store::find_repo_root(cwd)?;
            let request = models::MemorySaveRequest::from_reader(stdin)
                .context("failed to read memory save request JSON from stdin")?;
            let report = match command {
                MemoryCommand::Preview => {
                    store::preview_memory_save(&repo, &request.feature, &request.memory)?
                }
                MemoryCommand::Save => {
                    store::save_memory_with_report(&repo, &request.feature, &request.memory)?
                }
            };
            writeln!(stdout, "{}", serde_json::to_string_pretty(&report)?)?;
        }
        Command::Extract {
            feature,
            from,
            stdin: use_stdin,
            dry_run,
            json,
        } => {
            let repo = store::find_repo_root(cwd)?;
            let source_text = read_text_input(from.as_deref(), use_stdin, stdin)
                .context("failed to read extraction input")?;
            let config = config::Config::load(&repo).context("failed to load Promemo config")?;
            let provider = provider::provider_from_config(&config)?;
            let memory = provider.extract_memory(&provider::ExtractionRequest {
                feature: feature.clone(),
                source_text,
            })?;
            let report = if dry_run {
                store::preview_memory_save(&repo, &feature, &memory)?
            } else {
                store::save_memory_with_report(&repo, &feature, &memory)?
            };
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                write_save_report(stdout, &report, "Extracted memory")?;
            }
        }
        Command::SaveJson { feature, dry_run } => {
            let repo = store::find_repo_root(cwd)?;
            let memory = models::MemoryInput::from_reader(stdin)
                .context("failed to read memory JSON from stdin")?;
            let report = if dry_run {
                store::preview_memory_save(&repo, &feature, &memory)?
            } else {
                store::save_memory_with_report(&repo, &feature, &memory)?
            };
            write_save_report(stdout, &report, "Parsed memory")?;
        }
        Command::Save {
            feature,
            from,
            stdin: use_stdin,
            json,
            dry_run,
        } => {
            let repo = store::find_repo_root(cwd)?;
            let report = save_handoff(&repo, &feature, from.as_deref(), use_stdin, dry_run, stdin)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                write_save_report(stdout, &report, "Parsed handoff")?;
            }
        }
        Command::Load {
            feature,
            token_budget,
            related,
            json,
        } => {
            let repo = store::find_repo_root(cwd)?;
            let mut context = store::load_context(&repo, &feature)?;
            if let Some(budget) = token_budget {
                context.text = token_budget_text(&context.text, budget);
            }
            if related {
                append_related_features(&repo, &mut context, &feature)?;
            }
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
        Command::Search {
            query,
            semantic,
            hybrid,
            limit,
            json,
        } => {
            let repo = store::find_repo_root(cwd)?;
            let mode = if hybrid {
                search::SearchMode::Hybrid
            } else if semantic {
                search::SearchMode::Semantic
            } else {
                search::SearchMode::Keyword
            };
            let matches = search::search(&repo, &query, mode, limit)?;
            if json {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&matches)?)?;
            } else if matches.is_empty() {
                writeln!(stdout, "No memory found for \"{}\".", query)?;
            } else {
                for item in matches {
                    let score = item
                        .score
                        .map(|value| format!(" score={value:.3}"))
                        .unwrap_or_default();
                    writeln!(
                        stdout,
                        "{}:{}{}: {}",
                        item.path, item.line, score, item.snippet
                    )?;
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
        Command::Open { feature } => {
            let repo = store::find_repo_root(cwd)?;
            let path = store::feature_dir(&repo, &feature)?;
            if !path.exists() {
                bail!(
                    "feature `{}` does not exist; save it first with `promemo save {}` or `promemo save-json {}`",
                    feature,
                    feature,
                    feature
                );
            }
            open_path(&path)?;
            writeln!(stdout, "Opened {}", path.strip_prefix(&repo)?.display())?;
        }
        Command::Import { command } => match command {
            ImportCommand::Handoff {
                file,
                feature,
                json,
                dry_run,
            } => {
                let repo = store::find_repo_root(cwd)?;
                let report = save_handoff(&repo, &feature, Some(&file), false, dry_run, stdin)?;
                if json {
                    writeln!(stdout, "{}", serde_json::to_string_pretty(&report)?)?;
                } else {
                    write_save_report(stdout, &report, "Parsed handoff")?;
                }
            }
        },
    }

    Ok(())
}

fn write_cli_docs(stdout: &mut impl Write) -> Result<()> {
    let mut command = Cli::command();
    let help = command.render_long_help().to_string();
    writeln!(stdout, "# Promemo CLI Usage\n")?;
    writeln!(stdout, "Generated from `promemo --help`.\n")?;
    writeln!(stdout, "```txt\n{}\n```", help.trim_end())?;
    writeln!(stdout, "\n## Commands\n")?;
    for subcommand in command.get_subcommands_mut() {
        let name = subcommand.get_name().to_string();
        let help = subcommand.render_long_help().to_string();
        writeln!(stdout, "### `{}`\n", name)?;
        writeln!(stdout, "```txt\n{}\n```\n", help.trim_end())?;
    }
    Ok(())
}

fn append_related_features(
    repo: &Path,
    context: &mut models::LoadedContext,
    feature: &str,
) -> Result<()> {
    let related = search::related_features(repo, feature)?;
    if related.is_empty() {
        return Ok(());
    }
    let features = index::load_features(repo)?;
    let mut by_name = std::collections::BTreeMap::new();
    for item in features {
        by_name.insert(item.name.clone(), item);
    }
    context.text.push_str("\n\n## Related Features\n\n");
    for name in related {
        if let Some(item) = by_name.get(&name) {
            context
                .text
                .push_str(&format!("- `{}`: {}\n", item.name, item.summary));
        } else {
            context.text.push_str(&format!("- `{}`\n", name));
        }
    }
    Ok(())
}

fn token_budget_text(text: &str, budget: usize) -> String {
    if budget == 0 {
        return String::new();
    }
    let mut words = text.split_whitespace().collect::<Vec<_>>();
    if words.len() <= budget {
        return text.to_string();
    }
    words.truncate(budget);
    format!(
        "{}\n\n[Promemo truncated context to approximately {} tokens.]",
        words.join(" "),
        budget
    )
}

fn save_handoff(
    repo: &Path,
    feature: &str,
    from: Option<&Path>,
    use_stdin: bool,
    dry_run: bool,
    stdin: &mut impl Read,
) -> Result<models::SaveReport> {
    let input = read_text_input(from, use_stdin, stdin)?;
    let memory = handoff::parse_markdown(&input)?;
    if dry_run {
        store::preview_memory_save(repo, feature, &memory)
    } else {
        store::save_memory_with_report(repo, feature, &memory)
    }
}

fn read_text_input(from: Option<&Path>, use_stdin: bool, stdin: &mut impl Read) -> Result<String> {
    if use_stdin {
        let mut input = String::new();
        stdin
            .read_to_string(&mut input)
            .context("failed to read text from stdin")?;
        Ok(input)
    } else if let Some(path) = from {
        std::fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
    } else {
        anyhow::bail!("provide either `--from <file>` or `--stdin`");
    }
}

fn write_save_report(
    stdout: &mut impl Write,
    report: &models::SaveReport,
    parsed_label: &str,
) -> Result<()> {
    writeln!(stdout, "{}: {}", parsed_label, report.title)?;
    if report.dry_run {
        writeln!(stdout, "Dry run: no files were written")?;
        writeln!(stdout, "Would save feature: {}", report.feature)?;
        writeln!(stdout, "Would write {} files:", report.files_written.len())?;
    } else {
        writeln!(stdout, "Saved feature: {}", report.feature)?;
        writeln!(stdout, "Wrote {} files:", report.files_written.len())?;
    }

    for file in &report.files_written {
        writeln!(stdout, "  {}", file)?;
    }

    if !report.warnings.is_empty() {
        writeln!(stdout, "\nWarnings:")?;
        for warning in &report.warnings {
            writeln!(stdout, "  {}", warning)?;
        }
    }

    writeln!(stdout, "\nNext:")?;
    writeln!(stdout, "  promemo load {}", report.feature)?;
    writeln!(stdout, "  promemo search \"keyword\"")?;
    writeln!(stdout, "  promemo open {}", report.feature)?;
    writeln!(stdout, "\nView memory:")?;
    writeln!(
        stdout,
        "  less {}/features/{}/memory.md",
        store::MEMORY_DIR,
        report.feature
    )?;
    writeln!(
        stdout,
        "  open {}/features/{}",
        store::MEMORY_DIR,
        report.feature
    )?;
    Ok(())
}

fn open_path(path: &Path) -> Result<()> {
    let mut command = if cfg!(target_os = "macos") {
        let mut command = ProcessCommand::new("open");
        command.arg(path);
        command
    } else if cfg!(target_os = "windows") {
        let mut command = ProcessCommand::new("cmd");
        command.args(["/C", "start", ""]).arg(path);
        command
    } else {
        let mut command = ProcessCommand::new("xdg-open");
        command.arg(path);
        command
    };

    let status = command.status()?;
    if !status.success() {
        bail!("failed to open {}", path.display());
    }
    Ok(())
}
