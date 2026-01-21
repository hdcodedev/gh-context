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
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// GitHub URL or shorthand (owner/repo#number), or repo (owner/repo) in bulk mode
    pub input: String,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Md)]
    pub format: OutputFormat,

    /// Output to file
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Copy to clipboard (macos only for now via pbcopy if needed, or just print)
    /// Note: User requested --clip flag.
    #[arg(long)]
    pub clip: bool,

    /// Treat input as issue (disambiguate shorthand)
    #[arg(long)]
    pub issue: bool,

    /// Treat input as PR (disambiguate shorthand)
    #[arg(long)]
    pub pr: bool,

    /// Fetch multiple issues for a repo (list mode)
    #[arg(long)]
    pub bulk: bool,

    /// Issue state filter for bulk mode
    #[arg(long, value_enum, default_value_t = IssueState::Open)]
    pub state: IssueState,

    /// Items per page for bulk mode (1-100)
    #[arg(long, default_value_t = 30)]
    pub per_page: u32,

    /// Number of pages to fetch in bulk mode
    #[arg(long, default_value_t = 1)]
    pub pages: u32,
}
