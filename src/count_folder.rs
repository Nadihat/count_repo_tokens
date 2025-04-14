use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use tiktoken_rs::cl100k_base;
use walkdir::WalkDir;

/// Heuristic to detect binary files
fn is_binary(path: &PathBuf) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = [0u8; 8000];
        if let Ok(n) = file.read(&mut buffer) {
            return buffer[..n].contains(&0);
        }
    }
    true
}

/// Recursively collect all non-binary, readable files in a directory
fn get_all_files_in_dir(root: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|e| e.path().to_path_buf())
        .filter(|p| p.is_file() && !is_binary(p))
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run --release -- /path/to/directory");
        std::process::exit(1);
    }

    let dir_path = PathBuf::from(&args[1]);

    if !dir_path.exists() || !dir_path.is_dir() {
        eprintln!("Error: Path does not exist or is not a directory.");
        std::process::exit(1);
    }

    let tokenizer = cl100k_base().expect("Failed to load tokenizer");

    let files = get_all_files_in_dir(&dir_path);
    let mut total_tokens = 0;

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            let tokens = tokenizer.encode_ordinary(&content);
            total_tokens += tokens.len();
        }
    }

    println!("{}", total_tokens);
}
