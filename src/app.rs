use crate::cli::{Cli, Command, ImportCommand, SchemaCommand};
use crate::{git_snapshot, handoff, index, models, render, search, store};
use anyhow::{bail, Context, Result};
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
        Command::Schema { command } => match command {
            SchemaCommand::MemoryInput => {
                writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string_pretty(&models::memory_input_schema())?
                )?;
            }
        },
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
            } else if matches.is_empty() {
                writeln!(stdout, "No memory found for \"{}\".", query)?;
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

fn save_handoff(
    repo: &Path,
    feature: &str,
    from: Option<&Path>,
    use_stdin: bool,
    dry_run: bool,
    stdin: &mut impl Read,
) -> Result<models::SaveReport> {
    let input = if use_stdin {
        let mut input = String::new();
        stdin
            .read_to_string(&mut input)
            .context("failed to read handoff Markdown from stdin")?;
        input
    } else if let Some(path) = from {
        std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?
    } else {
        anyhow::bail!("provide either `--from <file>` or `--stdin`");
    };
    let memory = handoff::parse_markdown(&input)?;
    if dry_run {
        store::preview_memory_save(repo, feature, &memory)
    } else {
        store::save_memory_with_report(repo, feature, &memory)
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
