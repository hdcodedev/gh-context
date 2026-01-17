use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct GhAuthor {
    pub login: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GhComment {
    pub author: Option<GhAuthor>,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

// Raw output from `gh issue view` or `gh pr view`
#[derive(Debug, Deserialize)]
pub struct GhResponse {
    pub title: String,
    pub body: String,
    pub url: String,
    #[allow(dead_code)]
    pub number: u64,
    pub comments: Vec<GhComment>,
    #[serde(default)]
    pub author: Option<GhAuthor>,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub repo: String,
    pub number: u64,
    pub r#type: String, // "issue" or "pr"
    pub url: String,
    pub author: String,
}

#[derive(Debug, Serialize)]
pub struct UnifiedComment {
    pub author: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Context {
    pub metadata: Metadata,
    pub title: String,
    pub body: String,
    pub comments: Vec<UnifiedComment>,
    pub events: Vec<serde_json::Value>,
}
