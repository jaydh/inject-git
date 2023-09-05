use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

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
        "<a href=\"#[git]\"",
        &format!(
            "<a href=\"{}/blob/{}/{}\"",
            repo_url, current_branch, relative_path
        ),
    );

    let mut output_file = fs::File::create(file_path)?;
    output_file.write_all(modified_code.as_bytes())?;

    Ok(())
}

fn process_directory(dir_path: &str) -> io::Result<()> {
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().to_string_lossy().ends_with(".rs") {
            let current_branch = get_current_branch(&dir_path).unwrap();
            let relative_path = entry
                .path()
                .strip_prefix(dir_path)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap();

            inject_origin_url(
                &get_remote_origin_url(&dir_path).unwrap(),
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
