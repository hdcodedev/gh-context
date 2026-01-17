mod args;
mod format;
mod gh;
mod types;

#[cfg(test)]
mod __tests__;

use anyhow::{Context, Result};
use args::{Cli, OutputFormat};
use clap::Parser;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let target = gh::parse_target(&cli.input, cli.issue, cli.pr)?;
    let context = gh::fetch_context(&target)?;

    let formatted_output = match cli.format {
        OutputFormat::Json => format::to_json(&context)?,
        OutputFormat::Md => format::to_markdown(&context),
    };

    if let Some(path) = cli.out {
        fs::write(&path, &formatted_output)
            .with_context(|| format!("Failed to write output to file: {:?}", path))?;
    } else {
        println!("{}", formatted_output);
    }

    if cli.clip {
        // macOS 'pbcopy'
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn pbcopy for clipboard copy")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(formatted_output.as_bytes())
                .context("Failed to write to pbcopy stdin")?;
        }

        let status = child.wait().context("Failed to wait for pbcopy")?;
        if !status.success() {
            eprintln!("Warning: pbcopy exited with non-zero status");
        }
    }

    Ok(())
}
