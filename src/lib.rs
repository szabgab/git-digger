use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use once_cell::sync::Lazy;
use regex::Regex;

const URL_REGEXES: [&str; 2] = [
    "^https?://(github.com)/([^/]+)/([^/]+)/?.*$",
    "^https?://(gitlab.com)/([^/]+)/([^/]+)/?.*$",
];

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Repository {
    host: String,
    owner: String,
    repo: String,
}

#[allow(dead_code)]
impl Repository {
    fn new(host: &str, owner: &str, repo: &str) -> Self {
        Self {
            host: host.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
        }
    }

    /// Extracts the owner and repository name from a URL.
    ///
    /// Returns Repository
    ///
    /// Where host is either "github" or "gitlab" for now.
    ///
    /// e.g. https://github.com/szabgab/rust-digger -> ("github", "szabgab", "rust-digger")
    pub fn from_url(url: &str) -> Result<Self, Box<dyn Error>> {
        static REGS: Lazy<Vec<Regex>> = Lazy::new(|| {
            URL_REGEXES
                .iter()
                .map(|reg| Regex::new(reg).unwrap())
                .collect::<Vec<Regex>>()
        });

        for re in REGS.iter() {
            if let Some(repo_url) = re.captures(url) {
                let host = repo_url[1].to_lowercase();
                let owner = repo_url[2].to_lowercase();
                let repo = repo_url[3].to_lowercase();
                return Ok(Self { host, owner, repo });
            }
        }
        Err(format!("No match for repo in '{}'", &url).into())
    }

    pub fn url(&self) -> String {
        format!("https://{}/{}/{}", self.host, self.owner, self.repo)
    }

    pub fn path(&self, root: &Path) -> PathBuf {
        self.owner_path(root).join(&self.repo)
    }

    pub fn owner_path(&self, root: &Path) -> PathBuf {
        root.join(&self.host).join(&self.owner)
    }

    //let _ = git2::Repository::clone(repo, temp_dir_str);
    /// Run `git clone` or `git pull` to update a single repository
    pub fn update_repository(&self, root: &Path, clone: bool) -> Result<(), Box<dyn Error>> {
        let owner_path = self.owner_path(root);
        let current_dir = env::current_dir()?;
        log::info!(
            "Creating owner_path {:?} while current_dir is {:?}",
            &owner_path,
            &current_dir
        );
        fs::create_dir_all(&owner_path)?;
        let repo_path = self.path(root);
        if Path::new(&repo_path).exists() {
            if clone {
                log::info!("repo exist but we only clone now.  Skipping.");
            } else {
                log::info!("repo exist; cd to {:?}", &repo_path);
                env::set_current_dir(&repo_path)?;
                self.git_pull();
            }
        } else {
            log::info!("new repo; cd to {:?}", &owner_path);
            env::set_current_dir(owner_path)?;
            self.git_clone();
        }
        env::set_current_dir(current_dir)?;
        Ok(())
    }

    fn git_pull(&self) {
        let current_dir = env::current_dir().unwrap();
        log::info!("git pull in {current_dir:?}");

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
            Err(err) => {
                log::error!("Could not run git_pull in folder {current_dir:?} error: {err}")
            }
        }
    }

    fn git_clone(&self) {
        let current_dir = env::current_dir().unwrap();
        let url = self.url();
        log::info!("git clone {url} in {current_dir:?}");
        match Command::new("git").arg("clone").arg(self.url()).output() {
            Ok(result) => {
                if result.status.success() {
                    log::info!("git_clone exit code: '{}'", result.status);
                } else {
                    log::warn!(
                        "git_clone exit code: '{}' for url '{}' in '{current_dir:?}'",
                        result.status,
                        url,
                    );
                }
            }
            Err(err) => {
                log::error!("Could not run `git clone {url}` in {current_dir:?} error: {err}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_owner_and_repo() {
        let root = Path::new("/tmp");
        let expected = Repository::new("github.com", "szabgab", "rust-digger");

        // test https github.com, no slash at the end
        let repo = Repository::from_url("https://github.com/szabgab/rust-digger").unwrap();
        assert_eq!(repo, expected);
        assert_eq!(repo.url(), "https://github.com/szabgab/rust-digger");
        assert_eq!(
            repo.path(root).to_str(),
            Some("/tmp/github.com/szabgab/rust-digger")
        );

        // test http github.com trailing slash
        let repo = Repository::from_url("https://github.com/szabgab/rust-digger/").unwrap();
        assert_eq!(repo, expected);
        assert_eq!(repo.url(), "https://github.com/szabgab/rust-digger");

        // test http github.com trailing slash
        let repo = Repository::from_url("http://github.com/szabgab/rust-digger/").unwrap();
        assert_eq!(repo, expected);
        assert_eq!(repo.url(), "https://github.com/szabgab/rust-digger");

        // test https github.com link to a file
        let repo = Repository::from_url(
            "https://github.com/crypto-crawler/crypto-crawler-rs/tree/main/crypto-market-type",
        )
        .unwrap();
        assert_eq!(
            repo,
            Repository::new("github.com", "crypto-crawler", "crypto-crawler-rs",)
        );
        assert_eq!(
            repo.url(),
            "https://github.com/crypto-crawler/crypto-crawler-rs"
        );

        // test https gitlab.com
        let repo = Repository::from_url("https://gitlab.com/szabgab/rust-digger").unwrap();
        assert_eq!(
            repo,
            Repository::new("gitlab.com", "szabgab", "rust-digger")
        );
        assert_eq!(repo.url(), "https://gitlab.com/szabgab/rust-digger");

        // test converting to lowercase  gitlab.com
        let repo = Repository::from_url("https://gitlab.com/Szabgab/Rust-digger/").unwrap();
        assert_eq!(
            repo,
            Repository::new("gitlab.com", "szabgab", "rust-digger")
        );
        assert_eq!(repo.url(), "https://gitlab.com/szabgab/rust-digger");
        assert_eq!(repo.owner, "szabgab");
        assert_eq!(repo.repo, "rust-digger");
        assert_eq!(
            repo.path(root).to_str(),
            Some("/tmp/gitlab.com/szabgab/rust-digger")
        );

        // test incorrect URL
        let res = Repository::from_url("https://blabla.com/");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "No match for repo in 'https://blabla.com/'"
        );
    }
}
