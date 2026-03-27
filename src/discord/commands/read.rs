use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::discord::commands::switch::navigate;
use crate::discord::extract::{optional_string, optional_u64};
use crate::errors::AppResult;

#[derive(Debug, Deserialize, Serialize)]
struct MessageItem {
    author: String,
    time: String,
    message: String,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let server = optional_string(params, "server").unwrap_or("").to_string();
    let channel = optional_string(params, "channel").unwrap_or("").to_string();
    navigate(client, &server, &channel).await?;

    let count = optional_u64(params, "count", 20);

    let script = format!(
        r#"JSON.stringify((function(limit) {{
        var results = [];
        var msgNodes = document.querySelectorAll('[id^="chat-messages-"] > div, [class*="messageListItem"]');

        var slice = Array.from(msgNodes).slice(-limit);

        slice.forEach(function(node) {{
            var authorEl = node.querySelector('[class*="username"], [class*="headerText"] span');
            var timeEl = node.querySelector('time');
            var contentEl = node.querySelector('[id^="message-content-"], [class*="messageContent"]');

            if (contentEl) {{
                results.push({{
                    author: authorEl ? authorEl.textContent.trim() : '—',
                    time: timeEl ? (timeEl.getAttribute('datetime') || timeEl.textContent.trim()) : '',
                    message: (contentEl.textContent || '').trim().substring(0, 300),
                }});
            }}
        }});

        return results;
    }})({}))
    "#,
        count
    );

    let messages: Vec<MessageItem> = client.eval_json(&script).await?;

    if messages.is_empty() {
        return Ok(
            json!([{ "author": "System", "time": "", "message": "No messages found in the current channel." }]),
        );
    }
    Ok(serde_json::to_value(messages).unwrap_or(json!([])))
}
