mod args;
mod bulk;
mod format;
mod gh;
mod types;

#[cfg(test)]
mod __tests__;

use anyhow::{anyhow, Context, Result};
use args::{Cli, OutputFormat};
use bulk::{
    resolve_bulk_out_dir, validate_bulk_args,
};
use clap::Parser;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use types::Context as GhContext;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.bulk {
        validate_bulk_args(&cli)?;

        let repo_arg = match &cli.input {
            Some(input) => match gh::parse_repo(input) {
                Ok((owner, repo)) => format!("{}/{}", owner, repo),
                Err(_) => gh::detect_repo_from_git()?,
            },
            None => gh::detect_repo_from_git()?,
        };
        let parts: Vec<&str> = repo_arg.split('/').collect();
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let issue_numbers =
            gh::list_issue_numbers(&repo_arg, cli.state.as_str(), cli.per_page, cli.pages)?;

        if issue_numbers.is_empty() {
            println!("No issues found.");
            return Ok(());
        }

        let out_dir = resolve_bulk_out_dir(&cli)?;
        let file_extension = output_extension(&cli.format);

        for number in issue_numbers {
            let target = gh::Target {
                owner: owner.clone(),
                repo: repo.clone(),
                number,
                kind: gh::TargetType::Issue,
            };

            let context = gh::fetch_context(&target, true, false)?;
            let formatted_output = format_output(&context, &cli.format)?;

            let base = format!(
                "{}-{}-{}",
                repo, context.metadata.r#type, context.metadata.number
            );

            let file_path = out_dir.join(format!("{}.{}", base, file_extension));
            fs::write(&file_path, &formatted_output)
                .with_context(|| format!("Failed to write output to file: {:?}", file_path))?;
            println!("Generated context in {}", file_path.display());
        }

        return Ok(());
    }

    let target = match &cli.input {
        Some(input) => match gh::parse_target(input, cli.issue, false) {
            Ok(target) => target,
            Err(_) => {
                handle_default_mode(&cli)?;
                return Ok(());
            }
        },
        None => {
            handle_default_mode(&cli)?;
            return Ok(());
        }
    };

    let context = gh::fetch_context(&target, cli.issue, false)?;

    let formatted_output = format_output(&context, &cli.format)?;

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
            stdin
                .write_all(formatted_output.as_bytes())
                .context("Failed to write to pbcopy stdin")?;
        }

        let status = child.wait().context("Failed to wait for pbcopy")?;
        if !status.success() {
            eprintln!("Warning: pbcopy exited with non-zero status");
        }
    }

    Ok(())
}

fn handle_default_mode(cli: &Cli) -> Result<()> {
    let repo_arg = match &cli.input {
        Some(input) => match gh::parse_repo(input) {
            Ok((owner, repo)) => format!("{}/{}", owner, repo),
            Err(_) => gh::detect_repo_from_git()?,
        },
        None => gh::detect_repo_from_git()?,
    };

    let repo_arg = gh::resolve_effective_repo(&repo_arg)?;
    let issue_numbers =
        gh::list_issue_numbers(&repo_arg, cli.state.as_str(), cli.per_page, cli.pages)?;

    if issue_numbers.is_empty() {
        println!("No issues found.");
        return Ok(());
    }

    let parts: Vec<&str> = repo_arg.split('/').collect();
    let target = gh::Target {
        owner: parts[0].to_string(),
        repo: parts[1].to_string(),
        number: issue_numbers[0],
        kind: gh::TargetType::Issue,
    };

    let context = gh::fetch_context(&target, true, false)?;
    let formatted_output = format_output(&context, &cli.format)?;

    if let Some(path) = &cli.out {
        std::fs::write(path, &formatted_output)
            .with_context(|| format!("Failed to write output to file: {:?}", path))?;
    } else {
        println!("{}", formatted_output);
    }

    Ok(())
}

fn format_output(context: &GhContext, format: &OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => format::to_json(context),
        OutputFormat::Md => Ok(format::to_markdown(context)),
    }
}

fn output_extension(format: &OutputFormat) -> &'static str {
    match format {
        OutputFormat::Json => "json",
        OutputFormat::Md => "md",
    }
}
