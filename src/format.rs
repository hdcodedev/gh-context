use crate::types::Context;
use anyhow::{Context as _, Result};

pub fn to_json(context: &Context) -> Result<String> {
    serde_json::to_string_pretty(context).context("Failed to serialize context to JSON")
}

pub fn to_markdown(context: &Context) -> String {
    let mut md = String::new();

    // Title and URL
    md.push_str(&format!("# {}\n\n", context.title));
    md.push_str(&format!("URL: {}\n\n", context.metadata.url));

    // Body
    md.push_str("## Body\n\n");
    md.push_str(&context.body);
    md.push_str("\n\n");

    // Comments
    md.push_str("## Comments\n\n");
    for (i, comment) in context.comments.iter().enumerate() {
        md.push_str(&format!("### Comment {} by {}\n", i + 1, comment.author));
        if let Some(created_at) = &comment.created_at {
             md.push_str(&format!("_{}_\n", created_at));
        }
        md.push('\n');
        md.push_str(&comment.body);
        md.push_str("\n\n---\n\n");
    }

    // Timeline Events (Basic listing)
    md.push_str("## Timeline Events\n\n");
    for event in &context.events {
        if let Some(event_type) = event.get("event").and_then(|v| v.as_str()) {
            let actor = event.get("actor")
                .and_then(|a| a.get("login"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let created_at = event.get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            
            md.push_str(&format!("- **{}** by **{}** at {}\n", event_type, actor, created_at));
        }
    }

    md
}
