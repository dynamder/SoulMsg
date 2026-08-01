use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum PackageError {
    TomlParse(String),
    MissingPackageSection,
    MissingField(String),
    InvalidEdition(String),
    FileNotFound(String),
    IoError(std::io::Error),
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageError::TomlParse(msg) => write!(f, "TOML parse error: {}", msg),
            PackageError::MissingPackageSection => {
                write!(f, "Missing [package] section in package.toml")
            }
            PackageError::MissingField(field) => write!(f, "Missing required field: {}", field),
            PackageError::InvalidEdition(ed) => {
                write!(f, "Invalid edition: {}. Expected '2026'", ed)
            }
            PackageError::FileNotFound(path) => write!(f, "File not found: {}", path),
            PackageError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for PackageError {}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ImportError {
    InvalidPackageName(String),
    MalformedSyntax(String),
    UnresolvableImport(String),
    IoError(std::io::Error),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::InvalidPackageName(name) => write!(f, "Invalid package name: {}", name),
            ImportError::MalformedSyntax(msg) => write!(f, "Malformed import syntax: {}", msg),
            ImportError::UnresolvableImport(imp) => write!(f, "Cannot resolve import: {}", imp),
            ImportError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for ImportError {}

#[allow(dead_code)]
#[derive(Debug)]
pub enum HashError {
    ComputationFailed(String),
    ComparisonFailed(String),
    InvalidHashLength(usize),
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashError::ComputationFailed(msg) => {
                write!(f, "Hash computation failed: {}", msg)
            }
            HashError::ComparisonFailed(msg) => {
                write!(f, "Hash comparison failed: {}", msg)
            }
            HashError::InvalidHashLength(len) => {
                write!(f, "Invalid hash length: {} bytes (expected 32)", len)
            }
        }
    }
}

impl std::error::Error for HashError {}

#[derive(Debug)]
#[allow(dead_code)]
pub enum GitHubError {
    InvalidUrl(String),
    ApiError(String),
    HttpError(String),
    Base64DecodeError(String),
    FileNotFound(String),
    RateLimitExceeded,
}

impl fmt::Display for GitHubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitHubError::InvalidUrl(url) => write!(f, "Invalid GitHub URL: {}", url),
            GitHubError::ApiError(msg) => write!(f, "GitHub API error: {}", msg),
            GitHubError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            GitHubError::Base64DecodeError(msg) => write!(f, "Base64 decode error: {}", msg),
            GitHubError::FileNotFound(path) => write!(f, "File not found on GitHub: {}", path),
            GitHubError::RateLimitExceeded => write!(f, "GitHub API rate limit exceeded"),
        }
    }
}

impl std::error::Error for GitHubError {}
