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
    #[serde(rename = "hasIssuesEnabled")]
    pub has_issues_enabled: bool,
}

#[derive(Debug, serde::Deserialize)]
struct ParentRepoView {
    #[serde(rename = "hasIssuesEnabled")]
    pub has_issues_enabled: bool,
}

pub fn resolve_effective_repo(repo: &str) -> Result<String> {
    let output = Command::new("gh")
        .arg("repo")
        .arg("view")
        .arg(repo)
        .arg("--json")
        .arg("isFork,parent,nameWithOwner,hasIssuesEnabled")
        .output()
        .context("Failed to execute 'gh repo view' for repository metadata")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Could not read repository details. Make sure you're authenticated with GitHub CLI: {}",
            stderr.trim()
        ));
    }

    let repo_view: RepoView = serde_json::from_slice(&output.stdout)
        .context("Failed to parse JSON output from 'gh repo view'")?;

    if repo_view.is_fork {
        if let Some(parent) = repo_view.parent {
            // Check if parent repository actually has issues enabled before switching
            let parent_check = Command::new("gh")
                .arg("repo")
                .arg("view")
                .arg(&parent.name_with_owner)
                .arg("--json")
                .arg("hasIssuesEnabled")
                .output();

            if let Ok(parent_output) = parent_check {
                if parent_output.status.success() {
                    if let Ok(parent_repo) =
                        serde_json::from_slice::<ParentRepoView>(&parent_output.stdout)
                    {
                        if parent_repo.has_issues_enabled {
                            return Ok(parent.name_with_owner);
                        }
                    }
                }
            }
        }
    }

    // Verify even our current repo has issues enabled
    if !repo_view.has_issues_enabled {
        return Err(anyhow!("This repository has Issues disabled. Enable Issues in repository settings to use this tool."));
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
            return Err(anyhow!("This GitHub URL is not recognised. Use a full issue URL like: https://github.com/owner/repo/issues/123"));
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
            return Err(anyhow!(
                "Use the format owner/repo#123 for issue references."
            ));
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
        "Could not understand input. Provide either:\n• A full GitHub issue URL\n• Shorthand like owner/repo#123\nOr run without arguments to use the current repository."
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

fn try_git_remote_url(remote_name: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg(remote_name)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_git_remote_url(remote_name: &str) -> Result<String> {
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg(remote_name)
        .output()
        .with_context(|| {
            format!(
                "Failed to execute 'git remote get-url {}' for current repo detection",
                remote_name
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "'git remote get-url {}' failed: {}",
            remote_name,
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn detect_repo_from_git() -> Result<String> {
    if let Some(url) = try_git_remote_url("upstream") {
        if let Ok(repo) = parse_git_remote_url(&url) {
            return Ok(repo);
        }
    }

    let url = get_git_remote_url("origin")?;
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

    // gh issue view <number> --repo <owner>/<repo> --comments --json title,body,url,author,comments,number,assignees
    let output = Command::new("gh")
        .arg("issue")
        .arg("view")
        .arg(&num_arg)
        .arg("--repo")
        .arg(&repo_arg)
        .arg("--comments")
        .arg("--json")
        .arg("title,body,url,author,comments,number,assignees")
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

    // Check for linked open PRs
    let has_open_pr = events.iter().any(|event| {
        event.get("event") == Some(&serde_json::Value::String("cross-referenced".to_string()))
            && event
                .get("source")
                .and_then(|s| s.get("pull_request"))
                .and_then(|pr| pr.get("state"))
                == Some(&serde_json::Value::String("open".to_string()))
    });

    // Check if issue is assigned
    let is_assigned = gh_data
        .assignees
        .as_array()
        .map(|a| !a.is_empty())
        .unwrap_or(false);

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
        has_open_pr,
        is_assigned,
        confidence_score: 0,
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
