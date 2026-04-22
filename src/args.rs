use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, ValueEnum, Debug)]
pub enum OutputFormat {
    Json,
    Md,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum IssueState {
    Open,
    Closed,
    All,
}

impl IssueState {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueState::Open => "open",
            IssueState::Closed => "closed",
            IssueState::All => "all",
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Extract and triage actionable context from GitHub issues", long_about = None)]
pub struct Cli {
    /// GitHub issue URL or shorthand (owner/repo#number).
    /// If omitted, automatically find the best actionable issue in the current repository.
    pub input: Option<String>,

    /// Output format: Markdown for humans, JSON for scripts
    #[arg(long, value_enum, default_value_t = OutputFormat::Md)]
    pub format: OutputFormat,

    /// Write output to this file instead of printing
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Copy output directly to clipboard (macOS only)
    #[arg(long)]
    pub clip: bool,

    /// Treat input as issue (legacy flag, no effect)
    #[arg(long, hide = true)]
    pub issue: bool,

    /// Fetch all matching issues and save each to separate files
    #[arg(long)]
    pub bulk: bool,

    /// Filter issues by state in bulk mode
    #[arg(long, value_enum, default_value_t = IssueState::Open)]
    pub state: IssueState,

    /// Number of issues to fetch per page (1-100)
    #[arg(long, default_value_t = 30)]
    pub per_page: u32,

    /// Number of pages to fetch in bulk mode
    #[arg(long, default_value_t = 1)]
    pub pages: u32,
}
