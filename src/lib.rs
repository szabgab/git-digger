use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use once_cell::sync::Lazy;
use regex::Regex;

const URL_REGEXES: [&str; 2] = [
    "^https://(github).com/([^/]+)/([^/]+)/?.*$",
    "^https://(gitlab).com/([^/]+)/([^/]+)/?.*$",
];

/// Extracts the owner and repository name from a URL.
///
/// Returns a tuple of (host, owner, repo).
///
/// Where host is either "github" or "gitlab" for now.
///
/// e.g. https://github.com/szabgab/rust-digger -> ("github", "szabgab", "rust-digger")
pub fn get_owner_and_repo(repository: &str) -> (String, String, String) {
    static REGS: Lazy<Vec<Regex>> = Lazy::new(|| {
        URL_REGEXES
            .iter()
            .map(|reg| Regex::new(reg).unwrap())
            .collect::<Vec<Regex>>()
    });

    for re in REGS.iter() {
        if let Some(repo_url) = re.captures(repository) {
            let host = repo_url[1].to_lowercase();
            let owner = repo_url[2].to_lowercase();
            let repo = repo_url[3].to_lowercase();
            return (host, owner, repo);
        }
    }

    log::warn!("No match for repo in '{}'", &repository);
    (String::new(), String::new(), String::new())
}

/// Run `git clone` or `git pull` to update a single repository
pub fn update_single_repository(
    repos_folder: &Path,
    host: &str,
    owner: &str,
    repo: &str,
    repository_url: &str,
    clone: bool,
) -> Result<(), Box<dyn Error>> {
    let owner_path = repos_folder.join(host).join(owner);
    let current_dir = env::current_dir()?;
    log::info!(
        "Creating owner_path {:?} while current_dir is {:?}",
        &owner_path,
        &current_dir
    );
    fs::create_dir_all(&owner_path)?;
    let repo_path = owner_path.join(repo);
    if Path::new(&repo_path).exists() {
        if clone {
            log::info!("repo exist but we only clone now.  Skipping.");
        } else {
            log::info!("repo exist; cd to {:?}", &repo_path);
            env::set_current_dir(&repo_path)?;
            git_pull();
        }
    } else {
        log::info!("new repo; cd to {:?}", &owner_path);
        env::set_current_dir(owner_path)?;
        git_clone(repository_url, repo);
    }
    env::set_current_dir(current_dir)?;
    Ok(())
}

fn git_pull() {
    log::info!("git pull");
    let current_dir = env::current_dir().unwrap();

    match Command::new("git").arg("pull").output() {
        Ok(result) => {
            if result.status.success() {
                log::info!(
                    "git_pull exit code: '{}' in folder {:?}",
                    result.status,
                    current_dir
                );
            } else {
                log::warn!(
                    "git_pull exit code: '{}' in folder {:?}",
                    result.status,
                    current_dir
                );
            }
        }
        Err(err) => log::error!("Could not run git_pull in folder {current_dir:?} error: {err}"),
    }
}

fn git_clone(url: &str, path: &str) {
    log::info!("git clone {} {}", url, path);
    match Command::new("git").arg("clone").arg(url).arg(path).output() {
        Ok(result) => {
            if result.status.success() {
                log::info!("git_clone exit code: '{}'", result.status);
            } else {
                log::warn!(
                    "git_clone exit code: '{}' for url '{}' cloning to '{}'",
                    result.status,
                    url,
                    path
                );
            }
        }
        Err(err) => log::error!("Could not run git_clone {url} {path} error: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_owner_and_repo() {
        assert_eq!(
            get_owner_and_repo("https://github.com/szabgab/rust-digger"),
            (
                "github".to_string(),
                "szabgab".to_string(),
                "rust-digger".to_string()
            )
        );
        assert_eq!(
            get_owner_and_repo("https://github.com/szabgab/rust-digger/"),
            (
                "github".to_string(),
                "szabgab".to_string(),
                "rust-digger".to_string()
            )
        );
        assert_eq!(
            get_owner_and_repo(
                "https://github.com/crypto-crawler/crypto-crawler-rs/tree/main/crypto-market-type"
            ),
            (
                "github".to_string(),
                "crypto-crawler".to_string(),
                "crypto-crawler-rs".to_string()
            )
        );
        assert_eq!(
            get_owner_and_repo("https://gitlab.com/szabgab/rust-digger"),
            (
                "gitlab".to_string(),
                "szabgab".to_string(),
                "rust-digger".to_string()
            )
        );
        assert_eq!(
            get_owner_and_repo("https://gitlab.com/Szabgab/Rust-digger/"),
            (
                "gitlab".to_string(),
                "szabgab".to_string(),
                "rust-digger".to_string()
            )
        );
    }
}
