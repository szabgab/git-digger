//! # Git Digger
//!
//! A command-line tool for cloning and updating Git repositories.
//!
//! ## Usage
//!
//! ```bash
//! git-digger <repository_url> <root_folder>
//! ```
//!
//! ### Arguments
//!
//! - `repository_url`: The URL of the Git repository to clone or update
//! - `root_folder`: The local directory where the repository should be stored
//!
//! ### Examples
//!
//! Clone a repository from GitHub:
//! ```bash
//! git-digger https://github.com/user/repo.git /path/to/local/repos
//! ```
//!
//! Clone a repository from GitLab:
//! ```bash
//! git-digger https://gitlab.com/user/repo.git ~/projects
//! ```
//!
//! ### Behavior
//!
//! - If the repository doesn't exist locally, it will be cloned
//! - If the repository already exists, it will be updated
//! - The tool will create the necessary directory structure if it doesn't exist
//!
//! ### Exit Codes
//!
//! - `0`: Success
//! - `1`: Error (invalid arguments, repository error, or update failure)

/// Executable to be able to use the git-digger create as a command line tool.
///
/// Processes command-line arguments to clone or update a Git repository
/// in the specified root folder.
use git_digger::Repository;
use std::path::PathBuf;

fn main() {
    env_logger::init();
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        eprintln!("Usage: {} <repository_url> <root_folder>", args[0]);
        std::process::exit(1);
    }
    let repo_url = &args[1];
    let root = PathBuf::from(&args[2]);
    let clone = true;
    match Repository::from_url(repo_url) {
        Ok(repo) => match repo.update_repository(root.as_path(), clone, None) {
            Ok(_) => println!(
                "Repository updated successfully in {:?}",
                repo.path(root.as_path())
            ),
            Err(e) => {
                eprintln!("Error updating repository: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error creating repository from URL: {}", e);
            std::process::exit(1);
        }
    }
}
