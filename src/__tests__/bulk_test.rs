use crate::args::{Cli, IssueState, OutputFormat};
use crate::bulk::{
    resolve_bulk_out_dir, resolve_pr_range_out_dir, validate_bulk_args, validate_pr_range_args,
};
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
        from: None,
        to: None,
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

#[test]
fn test_validate_pr_range_args_ok() {
    let mut cli = make_cli();
    cli.bulk = false;
    cli.from = Some(10);
    cli.to = Some(12);
    assert_eq!(validate_pr_range_args(&cli).unwrap(), (10, 12));
}

#[test]
fn test_validate_pr_range_args_requires_both_bounds() {
    let mut cli = make_cli();
    cli.bulk = false;
    cli.from = Some(10);
    let err = validate_pr_range_args(&cli).unwrap_err();
    assert!(err.to_string().contains("provided together"));
}

#[test]
fn test_validate_pr_range_args_rejects_descending_range() {
    let mut cli = make_cli();
    cli.bulk = false;
    cli.from = Some(12);
    cli.to = Some(10);
    let err = validate_pr_range_args(&cli).unwrap_err();
    assert!(err.to_string().contains("less than or equal"));
}

#[test]
fn test_validate_pr_range_args_rejects_issue_flag() {
    let mut cli = make_cli();
    cli.bulk = false;
    cli.issue = true;
    cli.from = Some(1);
    cli.to = Some(2);
    let err = validate_pr_range_args(&cli).unwrap_err();
    assert!(err.to_string().contains("PRs only"));
}

#[test]
fn test_resolve_pr_range_out_dir_default() {
    let mut cli = make_cli();
    cli.bulk = false;
    cli.out = None;
    let dir = resolve_pr_range_out_dir(&cli, "repo").unwrap();
    assert_eq!(dir, PathBuf::from("repo-prs"));
    assert!(dir.is_dir());
    let _ = fs::remove_dir_all(&dir);
}
