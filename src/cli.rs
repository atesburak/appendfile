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
    if prefix.contains('/') || prefix.contains('\\') || prefix.contains("..") {
        return Err("prefix must not contain path separators or \"..\"".to_string());
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

#[cfg(test)]
mod tests {
    use super::{parse_cli, CliAction};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("appendfile-cli-test-{nanos}"))
    }

    #[test]
    fn parses_explicit_target_and_positionals() {
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("failed to create temp dir");
        let input = temp_dir.join("source.png");
        fs::write(&input, b"data").expect("failed to create input file");

        let args = vec![
            "--target".to_string(),
            temp_dir.to_string_lossy().to_string(),
            input.to_string_lossy().to_string(),
            "ayca".to_string(),
        ];

        let action = parse_cli(args.into_iter()).expect("expected successful parse");

        match action {
            CliAction::Run {
                input: parsed_input,
                prefix,
                target,
                persist_target,
            } => {
                assert_eq!(parsed_input, input);
                assert_eq!(prefix, "ayca");
                assert_eq!(target, temp_dir);
                assert!(persist_target);
            }
            CliAction::Help => panic!("unexpected help action"),
        }

        fs::remove_dir_all(temp_dir).expect("failed to clean up temp dir");
    }

    #[test]
    fn rejects_prefix_with_path_traversal() {
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("failed to create temp dir");
        let input = temp_dir.join("source.png");
        fs::write(&input, b"data").expect("failed to create input file");

        let args = vec![
            input.to_string_lossy().to_string(),
            "../evil".to_string(),
        ];

        let err = parse_cli(args.into_iter()).expect_err("expected parse error");
        assert!(err.contains("path separators"));

        fs::remove_dir_all(temp_dir).expect("failed to clean up temp dir");
    }

    #[test]
    fn rejects_missing_positional_arguments() {
        let err = parse_cli(vec!["input-only".to_string()].into_iter())
            .expect_err("expected parse error");
        assert!(err.contains("expected positional arguments"));
    }

    #[test]
    fn returns_help_for_help_flag() {
        let action =
            parse_cli(vec!["--help".to_string()].into_iter()).expect("expected help action");
        assert!(matches!(action, CliAction::Help));
    }
}
