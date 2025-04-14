use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::env;
use tiktoken_rs::cl100k_base;

/// Determines if a file is likely binary (quick heuristic).
fn is_binary(path: &PathBuf) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = [0u8; 8000];
        if let Ok(n) = file.read(&mut buffer) {
            return buffer[..n].contains(&0);
        }
    }
    true // if it can't be read, treat as binary
}

/// Use `git ls-files` to list tracked text files in a repo.
fn get_git_tracked_files(repo_path: &str) -> Vec<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("ls-files")
        .output()
        .expect("Failed to run git ls-files");

    if !output.status.success() {
        eprintln!("Git error: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    let files = String::from_utf8_lossy(&output.stdout);
    files
        .lines()
        .map(|line| PathBuf::from(repo_path).join(line.trim()))
        .filter(|path| path.is_file() && !is_binary(path))
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run --release -- /path/to/repo");
        std::process::exit(1);
    }

    let repo_path = &args[1];

    let tokenizer = cl100k_base().expect("Failed to load cl100k_base tokenizer");

    let files = get_git_tracked_files(repo_path);
    let mut total_tokens = 0;

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            let tokens = tokenizer.encode_ordinary(&content);
            total_tokens += tokens.len();
        }
    }

    println!("{}", total_tokens);
}
