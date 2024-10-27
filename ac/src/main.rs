use git2::Repository;
use std::error::Error;

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
    if args.len() < 2 {
        eprintln!("Usage: autocommitter <source_repo_path>");
        return;
    }

    let source_repo_path = &args[1];
    match get_commit_history(source_repo_path) {
        Ok(commits) => println!("Commits: {:?}", commits),
        Err(e) => println!("Error: {:?}", e),
    }
}
