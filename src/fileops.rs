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
