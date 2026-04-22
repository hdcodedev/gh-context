use crate::gh::{TargetType, parse_git_remote_url, parse_repo, parse_target};

#[test]
fn test_parse_full_url_issue() {
    let input = "https://github.com/rust-lang/rust/issues/123";
    let target = parse_target(input, true, false).unwrap();
    assert_eq!(target.owner, "rust-lang");
    assert_eq!(target.repo, "rust");
    assert_eq!(target.number, 123);
    assert!(matches!(target.kind, TargetType::Issue));
}

#[test]
fn test_parse_shorthand_issue() {
    let input = "rust-lang/rust#789";
    let target = parse_target(input, true, false).unwrap();
    assert_eq!(target.owner, "rust-lang");
    assert_eq!(target.repo, "rust");
    assert_eq!(target.number, 789);
    assert!(matches!(target.kind, TargetType::Issue));
}

#[test]
fn test_invalid_url() {
    let input = "https://github.com/rust-lang/rust/blob/main/README.md";
    let err = parse_target(input, true, false).unwrap_err();
    assert!(
        err.to_string()
            .contains("Failed to parse issue number from URL")
    );
}

#[test]
fn test_parse_full_url_with_fragment() {
    let input = "https://github.com/rust-lang/rust/issues/123#issuecomment-456";
    let target = parse_target(input, true, false).unwrap();
    assert_eq!(target.number, 123);
}

#[test]
fn test_parse_repo_owner_repo() {
    let input = "rust-lang/rust";
    let (owner, repo) = parse_repo(input).unwrap();
    assert_eq!(owner, "rust-lang");
    assert_eq!(repo, "rust");
}

#[test]
fn test_parse_repo_full_url_issues() {
    let input = "https://github.com/rust-lang/rust/issues";
    let (owner, repo) = parse_repo(input).unwrap();
    assert_eq!(owner, "rust-lang");
    assert_eq!(repo, "rust");
}

#[test]
fn test_parse_repo_full_url_issues_with_query() {
    let input = "https://github.com/rust-lang/rust/issues?state=open";
    let (owner, repo) = parse_repo(input).unwrap();
    assert_eq!(owner, "rust-lang");
    assert_eq!(repo, "rust");
}

#[test]
fn test_parse_repo_rejects_issue_number() {
    let input = "https://github.com/rust-lang/rust/issues/123";
    let err = parse_repo(input).unwrap_err();
    assert!(err.to_string().contains("issue number"));
}

#[test]
fn test_parse_git_remote_url_https() {
    let input = "https://github.com/rust-lang/rust.git";
    let repo = parse_git_remote_url(input).unwrap();
    assert_eq!(repo, "rust-lang/rust");
}

#[test]
fn test_parse_git_remote_url_ssh() {
    let input = "git@github.com:rust-lang/rust.git";
    let repo = parse_git_remote_url(input).unwrap();
    assert_eq!(repo, "rust-lang/rust");
}

#[test]
fn test_parse_git_remote_url_simple() {
    let input = "rust-lang/rust";
    let repo = parse_git_remote_url(input).unwrap();
    assert_eq!(repo, "rust-lang/rust");
}
