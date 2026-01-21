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
    let dir = if let Some(path) = &cli.out {
        if path.exists() && path.is_file() {
            return Err(anyhow!("--out must be a directory in bulk mode"));
        }
        path.clone()
    } else {
        PathBuf::from(format!("{}-issues", repo))
    };

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create output directory: {:?}", dir))?;
    }

    Ok(dir)
}
