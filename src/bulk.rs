use crate::args::Cli;
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;

pub fn validate_bulk_args(cli: &Cli) -> Result<()> {
    if cli.clip {
        return Err(anyhow!("--clip is not supported with --bulk"));
    }
    if cli.per_page == 0 || cli.per_page > 100 {
        return Err(anyhow!("--per-page must be between 1 and 100"));
    }
    if cli.pages == 0 {
        return Err(anyhow!("--pages must be at least 1"));
    }
    if cli.out.is_none() {
        return Err(anyhow!(
            "--out is required with --bulk to avoid writing in the current directory"
        ));
    }
    Ok(())
}

pub fn resolve_bulk_out_dir(cli: &Cli) -> Result<PathBuf> {
    resolve_out_dir(cli, "bulk mode")
}

fn resolve_out_dir(cli: &Cli, mode_label: &str) -> Result<PathBuf> {
    let dir = cli
        .out
        .clone()
        .ok_or_else(|| anyhow!("--out is required in {}", mode_label))?;

    if dir.exists() && dir.is_file() {
        return Err(anyhow!("--out must be a directory in {}", mode_label));
    }

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create output directory: {:?}", dir))?;
    }

    Ok(dir)
}
