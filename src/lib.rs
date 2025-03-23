use regex::Regex;
use once_cell::sync::Lazy;

const URL_REGEXES: [&str; 2] = [
    "^https://(github).com/([^/]+)/([^/]+)/?.*$",
    "^https://(gitlab).com/([^/]+)/([^/]+)/?.*$",
];


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

