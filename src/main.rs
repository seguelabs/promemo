use clap::Parser;
use promemo::cli::Cli;
use serde::Serialize;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let wants_json_errors = cli.wants_json_errors();
    let result = std::env::current_dir()
        .map_err(anyhow::Error::from)
        .and_then(|cwd| promemo::app::run(cli, &cwd));

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if wants_json_errors {
                let envelope = ErrorEnvelope::from_error(&error);
                eprintln!("{}", serde_json::to_string_pretty(&envelope).unwrap());
            } else {
                eprintln!("Error: {:#}", error);
            }
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl ErrorEnvelope {
    fn from_error(error: &anyhow::Error) -> Self {
        Self {
            error: ErrorBody {
                code: "promemo_error",
                message: error.to_string(),
            },
        }
    }
}
