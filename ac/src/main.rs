use git2::Repository;
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use std::thread::sleep;
use std::process::Command;

fn push_to_github(repo_path: &str, remote_url: &str) -> Result<(), Box<dyn Error>> {
    Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(remote_url)
        .output()?;

    Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg("main")
        .output()?;

    Ok(())
}

fn apply_commits_with_delay(
    source_commits: Vec<String>,
    new_repo: &Repository,
) -> Result<(), Box<dyn Error>> {
    for commit_message in source_commits {
        let mut index = new_repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = new_repo.find_tree(tree_oid)?;

        let author = new_repo.signature()?;
        let committer = new_repo.signature()?;

        new_repo.commit(
            Some("HEAD"),
            &author,
            &committer,
            &commit_message,
            &tree,
            &[],
        )?;
        println!("Commit added: {}", commit_message);

        sleep(Duration::from_secs(3600));  // 3600 seconds = 1 hour
    }

    Ok(())
}

fn initialize_new_repo(new_repo_path: &str) -> Result<Repository, Box<dyn Error>> {
    if Path::new(new_repo_path).exists() {
        println!("Target directory exists, using it...");
    } else {
        std::fs::create_dir_all(new_repo_path)?;
    }

    let new_repo = Repository::init(new_repo_path)?;
    Ok(new_repo)
}

fn get_commit_history(repo_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let repo = Repository::open(repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        commits.push(commit.summary().unwrap_or("").to_string());
    }

    Ok(commits)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: autocommitter <source_repo_path> <new_repo_path> <remote_url>");
        return;
    }

    let source_repo_path = &args[1];
    let new_repo_path = &args[2];
    let new_repo_url = &args[3];

    match get_commit_history(source_repo_path) {
        Ok(commits) => println!("Commits: {:?}", commits),
        Err(e) => println!("Error: {:?}", e),
    }

    match initialize_new_repo(new_repo_path) {
        Ok(_) => println!("New repository initialized."),
        Err(e) => println!("Error: {:?}", e),
    }
}
