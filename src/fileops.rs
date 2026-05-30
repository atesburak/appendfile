use std::fs;
use std::path::Path;

pub fn move_file(source: &Path, destination: &Path) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::move_file;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("appendfile-fileops-test-{nanos}"))
    }

    #[test]
    fn moves_file_with_rename() {
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("failed to create temp dir");

        let source = temp_dir.join("source.txt");
        let destination = temp_dir.join("destination.txt");
        fs::write(&source, b"hello").expect("failed to create source file");

        move_file(&source, &destination).expect("failed to move file");

        assert!(!source.exists());
        assert_eq!(
            fs::read(&destination).expect("failed to read destination"),
            b"hello"
        );

        fs::remove_dir_all(temp_dir).expect("failed to clean up temp dir");
    }

    #[test]
    fn fails_when_source_does_not_exist() {
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("failed to create temp dir");

        let source = temp_dir.join("missing.txt");
        let destination = temp_dir.join("destination.txt");
        let err = move_file(&source, &destination).expect_err("expected move failure");

        assert!(err.contains("failed to move"));

        fs::remove_dir_all(temp_dir).expect("failed to clean up temp dir");
    }
}
