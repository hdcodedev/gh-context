use crate::args::{Cli, IssueState, OutputFormat};
use crate::bulk::{resolve_bulk_out_dir, validate_bulk_args};
use std::fs;
use std::path::PathBuf;

fn make_cli() -> Cli {
    Cli {
        input: "owner/repo".to_string(),
        format: OutputFormat::Md,
        out: None,
        clip: false,
        issue: false,
        pr: false,
        bulk: true,
        state: IssueState::Open,
        per_page: 30,
        pages: 1,
    }
}

#[test]
fn test_validate_bulk_args_ok() {
    let cli = make_cli();
    assert!(validate_bulk_args(&cli).is_ok());
}

#[test]
fn test_validate_bulk_args_rejects_pr_flag() {
    let mut cli = make_cli();
    cli.pr = true;
    let err = validate_bulk_args(&cli).unwrap_err();
    assert!(err.to_string().contains("issues only"));
}

#[test]
fn test_validate_bulk_args_rejects_clip_flag() {
    let mut cli = make_cli();
    cli.clip = true;
    let err = validate_bulk_args(&cli).unwrap_err();
    assert!(err.to_string().contains("not supported"));
}

#[test]
fn test_validate_bulk_args_rejects_per_page_zero() {
    let mut cli = make_cli();
    cli.per_page = 0;
    let err = validate_bulk_args(&cli).unwrap_err();
    assert!(err.to_string().contains("per-page"));
}

#[test]
fn test_validate_bulk_args_rejects_per_page_overflow() {
    let mut cli = make_cli();
    cli.per_page = 101;
    let err = validate_bulk_args(&cli).unwrap_err();
    assert!(err.to_string().contains("per-page"));
}

#[test]
fn test_validate_bulk_args_rejects_zero_pages() {
    let mut cli = make_cli();
    cli.pages = 0;
    let err = validate_bulk_args(&cli).unwrap_err();
    assert!(err.to_string().contains("pages"));
}

#[test]
fn test_resolve_bulk_out_dir_creates_dir() {
    let mut cli = make_cli();
    let tmp_dir = std::env::temp_dir()
        .join(format!("gh-context-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    cli.out = Some(tmp_dir.clone());

    let resolved = resolve_bulk_out_dir(&cli, "repo").unwrap();
    assert_eq!(resolved, tmp_dir);
    assert!(resolved.is_dir());

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_resolve_bulk_out_dir_rejects_file() {
    let mut cli = make_cli();
    let tmp_file = std::env::temp_dir()
        .join(format!("gh-context-{}.txt", std::process::id()));
    let _ = fs::remove_file(&tmp_file);
    fs::write(&tmp_file, b"temp").unwrap();
    cli.out = Some(PathBuf::from(&tmp_file));

    let err = resolve_bulk_out_dir(&cli, "repo").unwrap_err();
    assert!(err.to_string().contains("directory"));

    let _ = fs::remove_file(&tmp_file);
}
