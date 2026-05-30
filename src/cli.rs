use std::env;
use std::path::PathBuf;

pub const USAGE: &str = "Usage: appendfile [--target <folder>] <input> <prefix>\n\nArguments:\n  <input>                Input media file (image or video)\n  <prefix>               Output file prefix\n\nOptions:\n  -t, --target <folder>  Set and remember target folder\n  -h, --help             Show this help message";

#[derive(Debug)]
pub enum CliAction {
    Run {
        input: PathBuf,
        prefix: String,
        target: PathBuf,
        persist_target: bool,
    },
    Help,
}

pub fn parse_cli<I>(mut args: I) -> Result<CliAction, String>
where
    I: Iterator<Item = String>,
{
    let mut target: Option<PathBuf> = None;
    let mut persist_target = false;
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(CliAction::Help),
            "-t" | "--target" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --target".to_string())?;
                target = Some(PathBuf::from(value));
                persist_target = true;
            }
            _ if arg.starts_with("--target=") => {
                let value = arg.trim_start_matches("--target=");
                if value.is_empty() {
                    return Err("missing value for --target".to_string());
                }
                target = Some(PathBuf::from(value));
                persist_target = true;
            }
            _ => positionals.push(arg),
        }
    }

    if positionals.len() != 2 {
        return Err("expected positional arguments: <input> <prefix>".to_string());
    }

    let input = PathBuf::from(&positionals[0]);
    if !input.is_file() {
        return Err(format!("input file not found: {}", input.display()));
    }

    let prefix = positionals[1].trim().to_string();
    if prefix.is_empty() {
        return Err("prefix cannot be empty".to_string());
    }

    let target = match target {
        Some(path) => path,
        None => match crate::storage::load_saved_target()? {
            Some(path) => path,
            None => env::current_dir()
                .map_err(|err| format!("failed to read current directory: {err}"))?,
        },
    };

    Ok(CliAction::Run {
        input,
        prefix,
        target,
        persist_target,
    })
}
