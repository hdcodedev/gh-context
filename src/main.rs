mod args;
mod bulk;
mod format;
mod gh;
mod types;

#[cfg(test)]
mod __tests__;

use anyhow::{Context, Result};
use args::{Cli, OutputFormat};
use bulk::{resolve_bulk_out_dir, validate_bulk_args};
use clap::Parser;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use types::Context as GhContext;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.bulk {
        validate_bulk_args(&cli)?;

        let (owner, repo) = gh::parse_repo(&cli.input)?;
        let repo_arg = format!("{}/{}", owner, repo);
        let issue_numbers =
            gh::list_issue_numbers(&repo_arg, cli.state.as_str(), cli.per_page, cli.pages)?;

        if issue_numbers.is_empty() {
            println!("No issues found.");
            return Ok(());
        }

        let out_dir = resolve_bulk_out_dir(&cli, &repo)?;
        let file_extension = output_extension(&cli.format);

        for number in issue_numbers {
            let target = gh::Target {
                owner: owner.clone(),
                repo: repo.clone(),
                number,
                kind: gh::TargetType::Issue,
            };

            let context = gh::fetch_context(&target)?;
            let formatted_output = format_output(&context, &cli.format)?;

            let base = format!(
                "{}-{}-{}",
                repo, context.metadata.r#type, context.metadata.number
            );

            let file_path = out_dir.join(format!("{}.{}", base, file_extension));
            fs::write(&file_path, &formatted_output).with_context(|| {
                format!("Failed to write output to file: {:?}", file_path)
            })?;
            println!("Generated context in {}", file_path.display());
        }

        return Ok(());
    }

    let target = gh::parse_target(&cli.input, cli.issue, cli.pr)?;
    let context = gh::fetch_context(&target)?;

    let formatted_output = format_output(&context, &cli.format)?;

    if let Some(path) = cli.out {
        fs::write(&path, &formatted_output)
            .with_context(|| format!("Failed to write output to file: {:?}", path))?;
    } else {
        match cli.format {
            OutputFormat::Json => {
                println!("{}", formatted_output);
            }
            OutputFormat::Md => {
                let repo_slug = context.metadata.repo
                    .split('/')
                    .nth(1)
                    .unwrap_or(&context.metadata.repo);
                let folder_name = format!(
                    "{}-{}-{}",
                    repo_slug, context.metadata.r#type, context.metadata.number
                );
                let folder_path = std::path::Path::new(&folder_name);
                if !folder_path.exists() {
                    fs::create_dir(folder_path).context("Failed to create directory")?;
                }

                let file_path = folder_path.join(format!("{}.md", folder_name));
                fs::write(&file_path, &formatted_output).with_context(|| {
                    format!("Failed to write output to file: {:?}", file_path)
                })?;
                println!("Generated context in {}", file_path.display());
            }
        }
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
