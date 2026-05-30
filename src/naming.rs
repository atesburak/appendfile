use std::fs;
use std::path::{Path, PathBuf};

pub fn next_output_path(target_dir: &Path, prefix: &str, input: &Path) -> Result<PathBuf, String> {
    let extension = input
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| format!("input file has no valid extension: {}", input.display()))?;
    let matching_extensions = matching_extensions_for(extension);

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

        if let Some(index) = parse_index(&file_name, prefix, &matching_extensions) {
            if index > max_index {
                max_index = index;
            }
        }
    }

    let next_index = max_index.saturating_add(1);
    let file_name = format!("{prefix}{next_index}.{extension}");
    Ok(target_dir.join(file_name))
}

fn parse_index(file_name: &str, prefix: &str, matching_extensions: &[String]) -> Option<u64> {
    let stem = file_name.strip_prefix(prefix)?;
    let stem = matching_extensions
        .iter()
        .find_map(|extension| stem.strip_suffix(&format!(".{extension}")))?;
    if stem.is_empty() {
        return None;
    }

    let index = stem.parse::<u64>().ok()?;
    if index == 0 {
        return None;
    }

    Some(index)
}

fn matching_extensions_for(extension: &str) -> Vec<String> {
    match extension.to_ascii_lowercase().as_str() {
        "png" => vec!["png".to_string(), "jpg".to_string()],
        "jpg" | "jpeg" => vec!["png".to_string(), "jpg".to_string(), "jpeg".to_string()],
        other => vec![other.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::next_output_path;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("appendfile-test-{nanos}"))
    }

    fn create_file(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent directory");
        }
        fs::write(path, b"data").expect("failed to create file");
    }

    #[test]
    fn next_output_path_uses_highest_png_or_jpg_index() {
        let target_dir = unique_temp_dir();
        fs::create_dir_all(&target_dir).expect("failed to create temp dir");
        create_file(&target_dir.join("ayca1.jpg"));
        create_file(&target_dir.join("ayca2.png"));
        create_file(&target_dir.join("ayca10.jpg"));
        create_file(&target_dir.join("ayca7.gif"));

        let output = next_output_path(&target_dir, "ayca", Path::new("/tmp/source.png"))
            .expect("failed to determine output path");

        assert_eq!(output, target_dir.join("ayca11.png"));

        fs::remove_dir_all(target_dir).expect("failed to clean up temp dir");
    }

    #[test]
    fn next_output_path_keeps_source_extension_when_family_is_mixed() {
        let target_dir = unique_temp_dir();
        fs::create_dir_all(&target_dir).expect("failed to create temp dir");
        create_file(&target_dir.join("photo1.png"));
        create_file(&target_dir.join("photo2.jpg"));

        let output = next_output_path(&target_dir, "photo", Path::new("/tmp/source.jpg"))
            .expect("failed to determine output path");

        assert_eq!(output, target_dir.join("photo3.jpg"));

        fs::remove_dir_all(target_dir).expect("failed to clean up temp dir");
    }
}