use crate::gh::{parse_target, TargetType};

#[test]
fn test_parse_full_url_issue() {
    let input = "https://github.com/rust-lang/rust/issues/123";
    let target = parse_target(input, false, false).unwrap();
    assert_eq!(target.owner, "rust-lang");
    assert_eq!(target.repo, "rust");
    assert_eq!(target.number, 123);
    matches!(target.kind, TargetType::Issue);
}

#[test]
fn test_parse_full_url_pr() {
    let input = "https://github.com/rust-lang/rust/pull/456";
    let target = parse_target(input, false, false).unwrap();
    assert_eq!(target.owner, "rust-lang");
    assert_eq!(target.repo, "rust");
    assert_eq!(target.number, 456);
    matches!(target.kind, TargetType::Pr);
}

#[test]
fn test_parse_shorthand_ambiguous() {
    let input = "rust-lang/rust#789";
    let err = parse_target(input, false, false).unwrap_err();
    assert!(err.to_string().contains("Ambiguous shorthand"));
}

#[test]
fn test_parse_shorthand_forced_issue() {
    let input = "rust-lang/rust#789";
    let target = parse_target(input, true, false).unwrap();
    assert_eq!(target.owner, "rust-lang");
    assert_eq!(target.repo, "rust");
    assert_eq!(target.number, 789);
    matches!(target.kind, TargetType::Issue);
}

#[test]
fn test_parse_shorthand_forced_pr() {
    let input = "rust-lang/rust#789";
    let target = parse_target(input, false, true).unwrap();
    assert_eq!(target.owner, "rust-lang");
    assert_eq!(target.repo, "rust");
    assert_eq!(target.number, 789);
    matches!(target.kind, TargetType::Pr);
}

#[test]
fn test_invalid_url() {
    let input = "https://github.com/rust-lang/rust/blob/main/README.md";
    let err = parse_target(input, false, false).unwrap_err();
    assert!(err.to_string().contains("URL must contain 'issues' or 'pull'"));
}
