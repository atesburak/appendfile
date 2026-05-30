use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

const APP_NAME: &str = "appendfile";
const TARGET_FILE: &str = "target";

pub fn run_with_target(
    input: PathBuf,
    prefix: String,
    target: PathBuf,
    persist_target: bool,
) -> Result<(), String> {
    if persist_target {
        save_target(&target)?;
    }

    fs::create_dir_all(&target).map_err(|err| {
        format!(
            "failed to create target directory {}: {err}",
            target.display()
        )
    })?;

    let output_path = crate::naming::next_output_path(&target, &prefix, &input)?;
    crate::fileops::move_file(&input, &output_path)?;

    println!("Input file: {}", input.display());
    println!("Output prefix: {prefix}");
    println!("Target folder: {}", target.display());
    println!("Created file: {}", output_path.display());

    Ok(())
}

pub fn save_target(target: &Path) -> Result<(), String> {
    save_target_to(target, None)
}

pub fn load_saved_target() -> Result<Option<PathBuf>, String> {
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
