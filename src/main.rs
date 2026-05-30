mod cli;
mod fileops;
mod naming;
mod storage;

use std::process;

use cli::CliAction;

fn main() {
    match cli::parse_cli(std::env::args().skip(1)) {
        Ok(CliAction::Help) => println!("{}", cli::USAGE),
        Ok(CliAction::Run {
            input,
            prefix,
            target,
            persist_target,
        }) => {
            if let Err(err) = storage::run_with_target(input, prefix, target, persist_target) {
                eprintln!("Error: {err}\n\n{}", cli::USAGE);
                process::exit(2);
            }
        }
        Err(message) => {
            eprintln!("Error: {message}\n\n{}", cli::USAGE);
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{cli, storage, CliAction};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn end_to_end_cli_moves_file_and_advances_mixed_image_numbering() {
        let target_dir = unique_temp_dir("appendfile-e2e-target");
        let work_dir = unique_temp_dir("appendfile-e2e-work");
        fs::create_dir_all(&target_dir).expect("failed to create target dir");
        fs::create_dir_all(&work_dir).expect("failed to create work dir");

        fs::write(target_dir.join("ayca1.jpg"), b"first").expect("failed to seed jpg file");
        fs::write(target_dir.join("ayca2.png"), b"second").expect("failed to seed png file");

        let input = work_dir.join("source.jpg");
        fs::write(&input, b"payload").expect("failed to create input file");

        let args = vec![
            "--target".to_string(),
            target_dir.to_string_lossy().to_string(),
            input.to_string_lossy().to_string(),
            "ayca".to_string(),
        ];

        let action = cli::parse_cli(args.into_iter()).expect("expected successful parse");

        match action {
            CliAction::Run {
                input,
                prefix,
                target,
                persist_target,
            } => {
                assert!(persist_target);
                storage::run_with_target_for_test(
                    input,
                    prefix,
                    target,
                    persist_target,
                    Some(&work_dir),
                )
                .expect("expected run to succeed");
            }
            CliAction::Help => panic!("unexpected help action"),
        }

        let moved_file = target_dir.join("ayca3.jpg");
        assert!(moved_file.exists());
        assert_eq!(
            fs::read(&moved_file).expect("failed to read moved file"),
            b"payload"
        );
        assert!(!input.exists());

        fs::remove_dir_all(target_dir).expect("failed to clean up target dir");
        fs::remove_dir_all(work_dir).expect("failed to clean up work dir");
    }
}
