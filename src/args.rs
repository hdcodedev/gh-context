use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, ValueEnum, Debug)]
pub enum OutputFormat {
    Json,
    Md,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// GitHub URL or shorthand (owner/repo#number)
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
}
