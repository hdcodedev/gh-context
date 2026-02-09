use crate::args::Cli;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn validate_bulk_args(cli: &Cli) -> Result<()> {
    if cli.pr {
        return Err(anyhow!("--bulk supports issues only; remove --pr"));
    }
    if cli.clip {
        return Err(anyhow!("--clip is not supported with --bulk"));
    }
    if cli.per_page == 0 || cli.per_page > 100 {
        return Err(anyhow!("--per-page must be between 1 and 100"));
    }
    if cli.pages == 0 {
        return Err(anyhow!("--pages must be at least 1"));
    }
    Ok(())
}

pub fn resolve_bulk_out_dir(cli: &Cli, repo: &str) -> Result<PathBuf> {
    resolve_out_dir(cli, format!("{}-issues", repo), "bulk mode")
}

pub fn validate_pr_range_args(cli: &Cli) -> Result<(u64, u64)> {
    if cli.bulk {
        return Err(anyhow!("--from/--to cannot be used with --bulk"));
    }
    if cli.issue {
        return Err(anyhow!("--from/--to supports PRs only; remove --issue"));
    }
    if cli.clip {
        return Err(anyhow!("--clip is not supported with --from/--to"));
    }

    let (from, to) = match (cli.from, cli.to) {
        (Some(from), Some(to)) => (from, to),
        (Some(_), None) | (None, Some(_)) => {
            return Err(anyhow!("--from and --to must be provided together"));
        }
        (None, None) => return Err(anyhow!("--from and --to are required for PR range mode")),
    };

    if from > to {
        return Err(anyhow!("--from must be less than or equal to --to"));
    }

    Ok((from, to))
}

pub fn resolve_pr_range_out_dir(cli: &Cli, repo: &str) -> Result<PathBuf> {
    resolve_out_dir(cli, format!("{}-prs", repo), "PR range mode")
}

fn resolve_out_dir(cli: &Cli, default_name: String, mode_label: &str) -> Result<PathBuf> {
    let dir = if let Some(path) = &cli.out {
        if path.exists() && path.is_file() {
            return Err(anyhow!("--out must be a directory in {}", mode_label));
        }
        path.clone()
    } else {
        PathBuf::from(default_name)
    };

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create output directory: {:?}", dir))?;
    }

    Ok(dir)
}
