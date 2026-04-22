use crate::types::{Context, GhResponse, Metadata, UnifiedComment};
use anyhow::{anyhow, Context as _, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum TargetType {
    Issue,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub kind: TargetType,
}

#[derive(Debug, serde::Deserialize)]
struct IssueListItem {
    pub number: u64,
}

#[derive(Debug, serde::Deserialize)]
struct RepoViewParent {
    #[serde(rename = "nameWithOwner")]
    pub name_with_owner: String,
}

#[derive(Debug, serde::Deserialize)]
struct RepoView {
    #[serde(rename = "isFork")]
    pub is_fork: bool,
    pub parent: Option<RepoViewParent>,
}

pub fn resolve_effective_repo(repo: &str) -> Result<String> {
    let output = Command::new("gh")
        .arg("repo")
        .arg("view")
        .arg("--repo")
        .arg(repo)
        .arg("--json")
        .arg("isFork,parent{nameWithOwner}")
        .output()
        .context("Failed to execute 'gh repo view' for repository metadata")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("'gh repo view' failed: {}", stderr));
    }

    let repo_view: RepoView = serde_json::from_slice(&output.stdout)
        .context("Failed to parse JSON output from 'gh repo view'")?;

    if repo_view.is_fork {
        if let Some(parent) = repo_view.parent {
            return Ok(parent.name_with_owner);
        }
    }

    Ok(repo.to_string())
}

pub fn parse_target(input: &str, _force_issue: bool, _force_pr: bool) -> Result<Target> {
    // case 1: Full URL
    if input.starts_with("https://github.com/") {
        let parts: Vec<&str> = input
            .trim_start_matches("https://github.com/")
            .split('/')
            .collect();
        if parts.len() < 4 {
            return Err(anyhow!("Invalid GitHub URL format"));
        }
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let number_str = parts[3];

        let number = number_str
            .split('#')
            .next()
            .unwrap()
            .split('?')
            .next()
            .unwrap()
            .parse::<u64>()
            .context("Failed to parse issue number from URL")?;

        return Ok(Target {
            owner,
            repo,
            number,
            kind: TargetType::Issue,
        });
    }

    // case 2: Shorthand owner/repo#number
    if let Some((repo_part, number_part)) = input.split_once('#') {
        let parts: Vec<&str> = repo_part.split('/').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Shorthand must be in format owner/repo#number"));
        }
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let number = number_part
            .parse::<u64>()
            .context("Failed to parse number from shorthand")?;

        return Ok(Target {
            owner,
            repo,
            number,
            kind: TargetType::Issue,
        });
    }

    Err(anyhow!(
        "Invalid input format. Must be a GitHub URL or owner/repo#number shorthand"
    ))
}

pub fn parse_repo(input: &str) -> Result<(String, String)> {
    if input.contains('#') {
        return Err(anyhow!("Repo input must not include an issue/pr number"));
    }

    let base = if let Some(rest) = input.strip_prefix("https://github.com/") {
        rest
    } else {
        input
    };
    let trimmed = base.split(|c| c == '?' || c == '#').next().unwrap_or("");

    let path = trimmed.trim_matches('/');
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        return Err(anyhow!("Repo input must be in format owner/repo"));
    }

    let owner = parts[0].to_string();
    let repo = parts[1].to_string();

    if parts.len() >= 3 {
        let segment = parts[2];
        if segment == "issues" {
            if parts.len() > 3 {
                return Err(anyhow!(
                    "Bulk issues URL should not include an issue number"
                ));
            }
        } else {
            return Err(anyhow!("Invalid repo URL format"));
        }
    }

    Ok((owner, repo))
}

pub fn detect_repo_from_git() -> Result<String> {
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .context("Failed to execute 'git remote get-url origin' for current repo detection")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("'git remote get-url' failed: {}", stderr));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_git_remote_url(&url)
}

pub fn parse_git_remote_url(url: &str) -> Result<String> {
    let url = url.trim().trim_end_matches(".git");

    if let Some(stripped) = url.strip_prefix("git@github.com:") {
        return parse_owner_repo(stripped);
    }

    if let Some(stripped) = url.strip_prefix("https://github.com/") {
        return parse_owner_repo(stripped);
    }

    if let Some(stripped) = url.strip_prefix("http://github.com/") {
        return parse_owner_repo(stripped);
    }

    if let Some(idx) = url.find("github.com/") {
        return parse_owner_repo(&url[idx + "github.com/".len()..]);
    }

    parse_owner_repo(url)
}

fn parse_owner_repo(path: &str) -> Result<String> {
    let trimmed = path.trim_matches('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Unsupported git remote URL format: {}", path));
    }
    Ok(format!("{}/{}", parts[0], parts[1]))
}

pub fn fetch_context(target: &Target, _force_issue: bool, _force_pr: bool) -> Result<Context> {
    let repo_arg = format!("{}/{}", target.owner, target.repo);
    let num_arg = target.number.to_string();

    // gh issue view <number> --repo <owner>/<repo> --comments --json title,body,url,author,comments,number
    let output = Command::new("gh")
        .arg("issue")
        .arg("view")
        .arg(&num_arg)
        .arg("--repo")
        .arg(&repo_arg)
        .arg("--comments")
        .arg("--json")
        .arg("title,body,url,author,comments,number")
        .output()
        .context("Failed to execute 'gh' command. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("'gh' command failed: {}", stderr));
    }

    let gh_data: GhResponse =
        serde_json::from_slice(&output.stdout).context("Failed to parse JSON output from 'gh'")?;

    // Convert to unified Context
    let comments: Vec<UnifiedComment> = gh_data
        .comments
        .into_iter()
        .map(|c| UnifiedComment {
            author: c
                .author
                .map(|a| a.login)
                .unwrap_or_else(|| "ghost".to_string()),
            body: c.body,
            created_at: c.created_at,
        })
        .collect();

    let author_login = gh_data
        .author
        .map(|a| a.login)
        .unwrap_or_else(|| "unknown".to_string());

    let events = fetch_timeline(target).unwrap_or_else(|_| Vec::new());

    let context = Context {
        metadata: Metadata {
            repo: repo_arg,
            number: target.number,
            r#type: "issue".to_string(),
            url: gh_data.url,
            author: author_login,
        },
        title: gh_data.title,
        body: gh_data.body,
        comments,
        events,
    };

    Ok(context)
}

pub fn list_issue_numbers(repo: &str, state: &str, per_page: u32, pages: u32) -> Result<Vec<u64>> {
    let limit = (per_page as u64) * (pages as u64);
    const MAX_ITEMS: u64 = 1000;
    if limit > MAX_ITEMS {
        return Err(anyhow!(
            "Requested {} items (per_page * pages) exceeds maximum allowed of {}",
            limit,
            MAX_ITEMS
        ));
    }

    let effective_repo = resolve_effective_repo(repo)?;
    let output = Command::new("gh")
        .arg("issue")
        .arg("list")
        .arg("--repo")
        .arg(&effective_repo)
        .arg("--state")
        .arg(state)
        .arg("--limit")
        .arg(limit.to_string())
        .arg("--json")
        .arg("number")
        .output()
        .context("Failed to execute 'gh issue list'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("'gh issue list' failed: {}", stderr));
    }

    let items: Vec<IssueListItem> = serde_json::from_slice(&output.stdout)
        .context("Failed to parse JSON output from 'gh issue list'")?;

    Ok(items.into_iter().map(|item| item.number).collect())
}

fn fetch_timeline(target: &Target) -> Result<Vec<serde_json::Value>> {
    let repo_arg = format!("{}/{}", target.owner, target.repo);
    let endpoint = format!("repos/{}/issues/{}/timeline", repo_arg, target.number);

    let output = Command::new("gh")
        .arg("api")
        .arg(&endpoint)
        .arg("--method")
        .arg("GET")
        .arg("--paginate")
        .output()
        .context("Failed to execute 'gh api' for timeline")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("'gh api' failed: {}", stderr));
    }

    let events: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .context("Failed to parse JSON output from 'gh api' timeline")?;

    Ok(events)
}
