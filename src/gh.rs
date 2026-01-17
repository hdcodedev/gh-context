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

pub fn parse_target(input: &str, force_issue: bool, force_pr: bool) -> Result<Target> {
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

