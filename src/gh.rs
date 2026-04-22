use crate::types::{Context, GhResponse, Metadata, UnifiedComment};
use anyhow::{anyhow, Context as _, Result};
use std::process::Command;

#[derive(Debug)]
pub enum TargetType {
    Issue,
    Pr,
}

#[derive(Debug)]
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

pub(crate) fn parse_repo_view_json(json: &str) -> Result<Option<String>> {
    let repo_view: RepoView = serde_json::from_str(json)
        .context("Failed to parse JSON output from 'gh repo view'")?;

    if repo_view.is_fork {
        Ok(repo_view.parent.map(|parent| parent.name_with_owner))
    } else {
        Ok(None)
    }
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

pub fn parse_target(input: &str, force_issue: bool, force_pr: bool) -> Result<Target> {
    if force_issue && force_pr {
        return Err(anyhow!("Cannot specify both --issue and --pr"));
    }

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
        let kind_str = parts[2];
        let number_str = parts[3];

        let kind = if kind_str == "issues" {
            TargetType::Issue
        } else if kind_str == "pull" {
            TargetType::Pr
        } else {
            return Err(anyhow!("URL must contain 'issues' or 'pull'"));
        };

        let number = number_str
            .split('#').next().unwrap()
            .split('?').next().unwrap()
            .parse::<u64>()
            .context("Failed to parse issue/pr number from URL")?;

        return Ok(Target {
            owner,
            repo,
            number,
            kind,
        });
    }

    // case 2: Shorthand owner/repo#number
    // We also support owner/repo issue_number if that's common, but strictly owner/repo#number is requested.
    // Actually, user said: <owner>/<repo>#<number>
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
        
        // Disambiguation
        let kind = if force_pr {
            TargetType::Pr
        } else if force_issue {
            TargetType::Issue
        } else {
             // If ambiguous, require --issue or --pr as per spec
             return Err(anyhow!("Ambiguous shorthand. Please specify --issue or --pr"));
        };

        return Ok(Target {
            owner,
            repo,
            number,
            kind,
        });
    }

    Err(anyhow!("Invalid input format. Must be a GitHub URL or owner/repo#number shorthand"))
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
                return Err(anyhow!("Bulk issues URL should not include an issue number"));
            }
        } else if segment == "pull" || segment == "pulls" {
            return Err(anyhow!("Bulk mode supports issues only; use an /issues URL"));
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

pub fn fetch_context(target: &Target) -> Result<Context> {
    let repo_arg = format!("{}/{}", target.owner, target.repo);
    let num_arg = target.number.to_string();

    let (subcommand, kind_str) = match target.kind {
        TargetType::Issue => ("issue", "issue"),
        TargetType::Pr => ("pr", "pr"),
    };

    // gh <subcommand> view <number> --repo <owner>/<repo> --comments --json title,body,url,author,comments
    let output = Command::new("gh")
        .arg(subcommand)
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

    let gh_data: GhResponse = serde_json::from_slice(&output.stdout)
        .context("Failed to parse JSON output from 'gh'")?;

    // Convert to unified Context
    let comments: Vec<UnifiedComment> = gh_data
        .comments
        .into_iter()
        .map(|c| UnifiedComment {
            author: c.author.map(|a| a.login).unwrap_or_else(|| "ghost".to_string()),
            body: c.body,
            created_at: c.created_at,
        })
        .collect();

    let author_login = gh_data.author.map(|a| a.login).unwrap_or_else(|| "unknown".to_string());

    let events = fetch_timeline(target).unwrap_or_else(|_| Vec::new());

    let context = Context {
        metadata: Metadata {
            repo: repo_arg,
            number: target.number,
            r#type: kind_str.to_string(),
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

pub fn list_issue_numbers(
    repo: &str,
    state: &str,
    per_page: u32,
    pages: u32,
) -> Result<Vec<u64>> {
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
        // Timeline might fail or be empty, strictly speaking we could return error
        // but for now let's just log it or return empty?
        // User asked for reliability. If it fails, maybe we should warn?
        // Let's generic error.
         let stderr = String::from_utf8_lossy(&output.stderr);
         return Err(anyhow!("'gh api' failed: {}", stderr));
    }

    let events: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .context("Failed to parse JSON output from 'gh api' timeline")?;

    Ok(events)
}
