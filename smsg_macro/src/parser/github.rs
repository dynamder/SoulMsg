use crate::error::GitHubError;
use base64::Engine;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubContentResponse {
    #[serde(rename = "type")]
    file_type: String,
    encoding: String,
    content: String,
    download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitHubDirEntry {
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
    download_url: Option<String>,
    size: u64,
}

#[derive(Debug, Clone)]
pub struct GitHubRepoInfo {
    pub owner: String,
    pub repo: String,
    pub reference: Option<String>,
}

pub fn parse_github_url(url: &str) -> Result<GitHubRepoInfo, GitHubError> {
    let url = url.trim().trim_end_matches('/');

    let host = if url.contains("github.com") {
        "github.com"
    } else {
        return Err(GitHubError::InvalidUrl(format!(
            "Not a GitHub URL: {}",
            url
        )));
    };

    let path_part = url.split(host).nth(1).unwrap_or("");
    let parts: Vec<&str> = path_part
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() < 2 {
        return Err(GitHubError::InvalidUrl(format!(
            "Invalid GitHub URL format: {}. Expected github.com/{{owner}}/{{repo}}",
            url
        )));
    }

    let owner = parts[0].to_string();
    let repo = parts[1].trim_end_matches(".git").to_string();

    Ok(GitHubRepoInfo {
        owner,
        repo,
        reference: None,
    })
}

pub fn parse_path_with_ref(path: &str) -> (Option<String>, &str) {
    let path = path.trim_start_matches('/');
    let hex_chars = "0123456789abcdefABCDEF";

    if path.len() >= 41
        && path
            .chars()
            .next()
            .map(|c| hex_chars.contains(c))
            .unwrap_or(false)
    {
        let first_40: String = path.chars().take(40).collect();
        let rest = &path[40..];

        if first_40.chars().all(|c| hex_chars.contains(c)) {
            if rest.is_empty() {
                return (Some(first_40), "");
            }
            let rest = rest.trim_start_matches('/');
            return (Some(first_40), rest);
        }
    }

    (None, path)
}

pub fn build_api_url(owner: &str, repo: &str, path: &str, reference: Option<&str>) -> String {
    let path = path.trim_start_matches('/');
    let base_url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        owner, repo, path
    );

    if let Some(r) = reference {
        format!("{}?ref={}", base_url, r)
    } else {
        base_url
    }
}

pub fn fetch_file_content(owner: &str, repo: &str, path: &str) -> Result<String, GitHubError> {
    fetch_file_content_at_ref(owner, repo, path, None)
}

pub fn fetch_file_content_at_ref(
    owner: &str,
    repo: &str,
    path: &str,
    reference: Option<&str>,
) -> Result<String, GitHubError> {
    let url = build_api_url(owner, repo, path, reference);

    let client = reqwest::blocking::Client::builder()
        .user_agent("SoulMsg/smsg_macro")
        .build()
        .map_err(|e| GitHubError::HttpError(e.to_string()))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| GitHubError::HttpError(e.to_string()))?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Err(GitHubError::FileNotFound(format!(
            "File not found on GitHub: {} (in {}/{})",
            path, owner, repo
        )));
    }

    if status.as_u16() == 403 {
        return Err(GitHubError::RateLimitExceeded);
    }

    if !status.is_success() {
        return Err(GitHubError::ApiError(format!(
            "GitHub API returned status {}",
            status.as_u16()
        )));
    }

    let content: GitHubContentResponse = response
        .json()
        .map_err(|e| GitHubError::ApiError(e.to_string()))?;

    if content.file_type != "file" {
        return Err(GitHubError::ApiError(format!(
            "Expected a file, got: {}",
            content.file_type
        )));
    }

    let decoded_content = if content.encoding == "base64" {
        let cleaned_content = content.content.lines().collect::<String>();
        base64::engine::general_purpose::STANDARD
            .decode(cleaned_content.as_bytes())
            .map_err(|e| GitHubError::Base64DecodeError(e.to_string()))?
    } else {
        return Err(GitHubError::ApiError(format!(
            "Unsupported encoding: {}",
            content.encoding
        )));
    };

    String::from_utf8(decoded_content)
        .map_err(|e| GitHubError::ApiError(format!("Invalid UTF-8 in file: {}", e)))
}

pub fn fetch_directory_entries(
    owner: &str,
    repo: &str,
    path: &str,
    reference: Option<&str>,
) -> Result<Vec<GitHubDirEntry>, GitHubError> {
    let url = build_api_url(owner, repo, path, reference);

    let client = reqwest::blocking::Client::builder()
        .user_agent("SoulMsg/smsg_macro")
        .build()
        .map_err(|e| GitHubError::HttpError(e.to_string()))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| GitHubError::HttpError(e.to_string()))?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Err(GitHubError::FileNotFound(format!(
            "Directory not found on GitHub: {} (in {}/{})",
            path, owner, repo
        )));
    }

    if status.as_u16() == 403 {
        return Err(GitHubError::RateLimitExceeded);
    }

    if !status.is_success() {
        return Err(GitHubError::ApiError(format!(
            "GitHub API returned status {}",
            status.as_u16()
        )));
    }

    let entries: Vec<GitHubDirEntry> = response
        .json()
        .map_err(|e| GitHubError::ApiError(e.to_string()))?;

    Ok(entries)
}

pub fn fetch_directory_contents_recursive(
    owner: &str,
    repo: &str,
    base_path: &str,
    reference: Option<&str>,
) -> Result<HashMap<String, String>, GitHubError> {
    let mut files = HashMap::new();
    fetch_directory_recursive_impl(owner, repo, base_path, &mut files, reference)?;
    Ok(files)
}

fn fetch_directory_recursive_impl(
    owner: &str,
    repo: &str,
    path: &str,
    files: &mut HashMap<String, String>,
    reference: Option<&str>,
) -> Result<(), GitHubError> {
    let entries = fetch_directory_entries(owner, repo, path, reference)?;

    for entry in entries {
        let entry_path = if path.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", path, entry.name)
        };

        match entry.entry_type.as_str() {
            "file" => {
                if entry.name.ends_with(".smsg") || entry.name == "package.toml" {
                    if let Some(download_url) = entry.download_url {
                        let content = fetch_content_from_url(&download_url)?;
                        files.insert(entry_path, content);
                    }
                }
            }
            "dir" => {
                fetch_directory_recursive_impl(owner, repo, &entry_path, files, reference)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn fetch_content_from_url(url: &str) -> Result<String, GitHubError> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("SoulMsg/smsg_macro")
        .build()
        .map_err(|e| GitHubError::HttpError(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| GitHubError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(GitHubError::HttpError(format!(
            "Failed to download: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .map_err(|e| GitHubError::HttpError(e.to_string()))?;

    String::from_utf8(bytes.to_vec())
        .map_err(|e| GitHubError::ApiError(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_standard() {
        let result = parse_github_url("https://github.com/owner/repo").unwrap();
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
    }

    #[test]
    fn test_parse_github_url_with_trailing_slash() {
        let result = parse_github_url("https://github.com/owner/repo/").unwrap();
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
    }

    #[test]
    fn test_parse_github_url_git_suffix() {
        let result = parse_github_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
    }

    #[test]
    fn test_parse_github_url_http() {
        let result = parse_github_url("http://github.com/owner/repo").unwrap();
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
    }

    #[test]
    fn test_parse_github_url_invalid() {
        assert!(parse_github_url("https://gitlab.com/owner/repo").is_err());
        assert!(parse_github_url("https://github.com/owner").is_err());
        assert!(parse_github_url("not-a-url").is_err());
    }

    #[test]
    fn test_build_api_url() {
        let url = build_api_url("owner", "repo", "path/to/file.smsg", None);
        assert_eq!(
            url,
            "https://api.github.com/repos/owner/repo/contents/path/to/file.smsg"
        );
    }

    #[test]
    fn test_build_api_url_with_leading_slash() {
        let url = build_api_url("owner", "repo", "/path/to/file.smsg", None);
        assert_eq!(
            url,
            "https://api.github.com/repos/owner/repo/contents/path/to/file.smsg"
        );
    }

    #[test]
    fn test_build_api_url_with_ref() {
        let url = build_api_url(
            "owner",
            "repo",
            "path/to/file.smsg",
            Some("06a7fcd9e2b0edfa6b6a37bc6f42824b5238a9c1"),
        );
        assert_eq!(
            url,
            "https://api.github.com/repos/owner/repo/contents/path/to/file.smsg?ref=06a7fcd9e2b0edfa6b6a37bc6f42824b5238a9c1"
        );
    }

    #[test]
    fn test_parse_path_with_ref() {
        let result = parse_path_with_ref(
            "06a7fcd9e2b0edfa6b6a37bc6f42824b5238a9c1/tests/fixtures/messages.smsg",
        );
        assert_eq!(
            result.0,
            Some("06a7fcd9e2b0edfa6b6a37bc6f42824b5238a9c1".to_string())
        );
        assert_eq!(result.1, "tests/fixtures/messages.smsg");
    }

    #[test]
    fn test_parse_path_with_ref_no_ref() {
        let result = parse_path_with_ref("tests/fixtures/messages.smsg");
        assert_eq!(result.0, None);
        assert_eq!(result.1, "tests/fixtures/messages.smsg");
    }
}
