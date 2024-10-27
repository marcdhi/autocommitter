use git2::{Repository, Commit, DiffOptions, DiffDelta, Delta, Oid};
use std::error::Error;
use std::path::Path;
use std::fs;
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
    source_repo_path: &str,
    source_commits: Vec<Oid>,
    new_repo: &Repository,
    new_repo_path: &str,
    remote_url: &str,
) -> Result<(), Box<dyn Error>> {
    let source_repo = Repository::open(source_repo_path)?;
    let mut parent_commit: Option<Commit> = None;

    for commit_oid in source_commits {
        // Find the commit in the source repository
        let commit = source_repo.find_commit(commit_oid)?;
        let commit_message = commit.message().unwrap_or("").to_string();
        
        // Get the diff between this commit and its parent
        let diff = if let Some(parent) = commit.parent(0).ok() {
            source_repo.diff_tree_to_tree(Some(&parent.tree()?), Some(&commit.tree()?), Some(&mut DiffOptions::new()))?
        } else {
            source_repo.diff_tree_to_tree(None, Some(&commit.tree()?), Some(&mut DiffOptions::new()))?
        };

        // Apply the changes to the new repository
        diff.foreach(&mut |delta: DiffDelta, _| {
            if let Some(new_file) = delta.new_file().path() {
                let path = new_repo_path.to_string() + "/" + new_file.to_str().unwrap();
                match delta.status() {
                    Delta::Added | Delta::Modified => {
                        if let Ok(blob) = commit.tree().and_then(|tree| tree.get_path(new_file)) {
                            if let Ok(object) = blob.to_object(&source_repo) {
                                if let Some(content) = object.as_blob() {
                                    fs::write(&path, content.content()).unwrap();
                                }
                            }
                        }
                    },
                    Delta::Deleted => {
                        fs::remove_file(&path).unwrap_or(());
                    },
                    _ => {}
                }
            }
            true
        }, None, None, None)?;

        // Stage all changes
        let mut index = new_repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // Commit the changes
        let tree_id = index.write_tree()?;
        let tree = new_repo.find_tree(tree_id)?;
        let signature = new_repo.signature()?;
        
        new_repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &commit_message,
            &tree,
            &parent_commit.iter().collect::<Vec<_>>(),
        )?;

        println!("Commit added with changes: {}", commit_message);

        // Push the commit immediately after creating it
        push_to_github(new_repo_path, remote_url)?;
        println!("Commit pushed to remote repository");

        // Update parent_commit for the next iteration
        parent_commit = Some(new_repo.head()?.peel_to_commit()?);

        // sleep(Duration::from_secs(3600));  // 3600 seconds = 1 hour
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

fn get_commit_history(repo_path: &str) -> Result<Vec<Oid>, Box<dyn Error>> {
    let repo = Repository::open(repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;

    let commits: Result<Vec<Oid>, _> = revwalk.collect();
    Ok(commits?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: autocommitter <source_repo_path> <new_repo_path> <remote_url>");
        return Ok(());
    }

    let source_repo_path = &args[1];
    let new_repo_path = &args[2];
    let new_repo_url = &args[3];

    let source_commits = get_commit_history(source_repo_path)?;
    let new_repo = initialize_new_repo(new_repo_path)?;

    // Set up the remote before applying commits
    Command::new("git")
        .arg("-C")
        .arg(new_repo_path)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(new_repo_url)
        .output()?;

    apply_commits_with_delay(source_repo_path, source_commits, &new_repo, new_repo_path, new_repo_url)?;

    println!("AutoCommitter: Successfully recreated and pushed commits with changes to the new repository.");
    Ok(())
}
