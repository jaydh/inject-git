use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use walkdir::WalkDir;

fn get_git_repository_root(dir_path: &str) -> io::Result<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(dir_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    if output.status.success() {
        let root_path = String::from_utf8_lossy(&output.stdout);
        Ok(root_path.trim().to_string())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to determine Git repository root",
        ))
    }
}

fn get_remote_origin_url(dir_path: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .current_dir(dir_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .ok()?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout);
        Some(url.trim().to_string())
    } else {
        None
    }
}

fn get_current_branch(dir_path: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(dir_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .ok()?;

    if output.status.success() {
        let branch_name = String::from_utf8_lossy(&output.stdout);
        Some(branch_name.trim().to_string())
    } else {
        None
    }
}

fn inject_origin_url(
    repo_url: &str,
    current_branch: &str,
    file_path: &str,
    relative_path: &str,
) -> io::Result<()> {
    let mut input_file = fs::File::open(file_path)?;
    let mut code = String::new();
    input_file.read_to_string(&mut code)?;

    let modified_code = code.replace(
        "#[git]",
        &format!("{}/tree/{}/{}", repo_url, current_branch, relative_path),
    );

    let mut output_file = fs::File::create(file_path)?;
    output_file.write_all(modified_code.as_bytes())?;

    Ok(())
}

fn process_directory(dir_path: &str) -> io::Result<()> {
    let root_dir = get_git_repository_root(dir_path)?;

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().to_string_lossy().ends_with(".rs") {
            let current_branch = get_current_branch(&dir_path).unwrap();
            let full_path: PathBuf = fs::canonicalize(entry.path())?;
            let relative_path = full_path
                .strip_prefix(&root_dir)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap();
            let remote_origin_url = get_remote_origin_url(&dir_path).unwrap();

            println!(
                "Processing {} with remote_origin_url={}, current_branch={}",
                relative_path, remote_origin_url, current_branch
            );

            inject_origin_url(
                &remote_origin_url.strip_suffix(".git").unwrap(),
                &current_branch,
                &entry.path().to_str().unwrap(),
                &relative_path,
            )?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <dir_path>", &args[0]);
        return Ok(());
    }

    let dir_path = &args[1];

    process_directory(dir_path)?;

    Ok(())
}
