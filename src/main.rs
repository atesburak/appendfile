use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;

const USAGE: &str = "Usage: appendfile [--target <folder>] <input> <prefix>\n\nArguments:\n  <input>                Input media file (image or video)\n  <prefix>               Output file prefix\n\nOptions:\n  -t, --target <folder>  Set and remember target folder\n  -h, --help             Show this help message";
const APP_NAME: &str = "appendfile";
const TARGET_FILE: &str = "target";

#[derive(Debug)]
enum CliAction {
    Run {
        input: PathBuf,
        prefix: String,
        target: PathBuf,
        persist_target: bool,
    },
    Help,
}

fn main() {
    match parse_cli(env::args().skip(1)) {
        Ok(CliAction::Help) => println!("{USAGE}"),
        Ok(CliAction::Run {
            input,
            prefix,
            target,
            persist_target,
        }) => {
            if persist_target {
                if let Err(err) = save_target(&target) {
                    eprintln!("Error: {err}\n\n{USAGE}");
                    process::exit(2);
                }
            }

            if let Err(err) = fs::create_dir_all(&target) {
                eprintln!(
                    "Error: failed to create target directory {}: {err}\n\n{USAGE}",
                    target.display()
                );
                process::exit(2);
            }

            let output_path = match next_output_path(&target, &prefix, &input) {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("Error: {err}\n\n{USAGE}");
                    process::exit(2);
                }
            };

            if let Err(err) = move_file(&input, &output_path) {
                eprintln!("Error: {err}\n\n{USAGE}");
                process::exit(2);
            }

            println!("Input file: {}", input.display());
            println!("Output prefix: {prefix}");
            println!("Target folder: {}", target.display());
            println!("Created file: {}", output_path.display());
        }
        Err(message) => {
            eprintln!("Error: {message}\n\n{USAGE}");
            process::exit(2);
        }
    }
}

fn parse_cli<I>(mut args: I) -> Result<CliAction, String>
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
        None => match load_saved_target()? {
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

fn save_target(target: &Path) -> Result<(), String> {
    save_target_to(target, None)
}

fn next_output_path(target_dir: &Path, prefix: &str, input: &Path) -> Result<PathBuf, String> {
    let extension = input
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| format!("input file has no valid extension: {}", input.display()))?;

    let mut max_index: u64 = 0;
    let entries = fs::read_dir(target_dir).map_err(|err| {
        format!(
            "failed to read target directory {}: {err}",
            target_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|err| {
            format!(
                "failed to read entry in target directory {}: {err}",
                target_dir.display()
            )
        })?;

        let file_name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        if let Some(index) = parse_index(&file_name, prefix, extension) {
            if index > max_index {
                max_index = index;
            }
        }
    }

    let next_index = max_index.saturating_add(1);
    let file_name = format!("{prefix}{next_index}.{extension}");
    Ok(target_dir.join(file_name))
}

fn parse_index(file_name: &str, prefix: &str, extension: &str) -> Option<u64> {
    let suffix = format!(".{extension}");
    let stem = file_name.strip_prefix(prefix)?.strip_suffix(&suffix)?;
    if stem.is_empty() {
        return None;
    }

    let index = stem.parse::<u64>().ok()?;
    if index == 0 {
        return None;
    }

    Some(index)
}

fn move_file(source: &Path, destination: &Path) -> Result<(), String> {
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(rename_err) if is_cross_device_rename_error(&rename_err) => {
            fs::copy(source, destination).map_err(|copy_err| {
                format!(
                    "failed to move {} to {}: rename failed ({rename_err}), copy failed ({copy_err})",
                    source.display(),
                    destination.display()
                )
            })?;

            fs::remove_file(source).map_err(|remove_err| {
                format!(
                    "failed to remove original file {} after copy to {}: {remove_err}",
                    source.display(),
                    destination.display()
                )
            })
        }
        Err(err) => Err(format!(
            "failed to move {} to {}: {err}",
            source.display(),
            destination.display()
        )),
    }
}

fn is_cross_device_rename_error(err: &std::io::Error) -> bool {
    err.raw_os_error() == Some(18)
}

fn load_saved_target() -> Result<Option<PathBuf>, String> {
    load_saved_target_from(None)
}

fn save_target_to(target: &Path, config_root_override: Option<&Path>) -> Result<(), String> {
    let target_file = target_file_path(config_root_override)?;
    if let Some(parent) = target_file.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create config directory {}: {err}",
                parent.display()
            )
        })?;
    }

    fs::write(&target_file, target.to_string_lossy().as_bytes())
        .map_err(|err| format!("failed to save target to {}: {err}", target_file.display()))
}

fn load_saved_target_from(config_root_override: Option<&Path>) -> Result<Option<PathBuf>, String> {
    let target_file = target_file_path(config_root_override)?;
    let raw = match fs::read_to_string(&target_file) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(format!(
                "failed to read saved target from {}: {err}",
                target_file.display()
            ))
        }
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(PathBuf::from(trimmed)))
}

fn target_file_path(config_root_override: Option<&Path>) -> Result<PathBuf, String> {
    let mut path = config_root(config_root_override)?;
    path.push(APP_NAME);
    path.push(TARGET_FILE);
    Ok(path)
}

fn config_root(config_root_override: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = config_root_override {
        return Ok(path.to_path_buf());
    }

    if let Ok(path) = env::var("XDG_CONFIG_HOME") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    if let Ok(home) = env::var("HOME") {
        if !home.is_empty() {
            let mut path = PathBuf::from(home);
            path.push(".config");
            return Ok(path);
        }
    }

    Err("could not determine config directory (XDG_CONFIG_HOME or HOME)".to_string())
}
