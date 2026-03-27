use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;

#[derive(Debug, Deserialize, Serialize)]
struct MemberItem {
    index: u32,
    name: String,
    status: String,
}

pub async fn execute(client: &AgentBrowserClient, _params: &Value) -> AppResult<Value> {
    let script = r#"JSON.stringify((function() {
        var results = [];
        var items = document.querySelectorAll('[class*="member_"], [data-list-item-id*="members"]');

        items.forEach(function(item, i) {
            var nameEl = item.querySelector('[class*="username_"], [class*="nameTag"]');
            var statusEl = item.querySelector('[class*="activity"], [class*="customStatus"]');

            var name = nameEl ? nameEl.textContent.trim() : (item.textContent || '').trim().substring(0, 50);
            var status = statusEl ? statusEl.textContent.trim() : '';

            if (name && name.length > 0) {
                results.push({ index: i + 1, name: name, status: status || 'Online' });
            }
        });

        return results.slice(0, 50);
    })())"#;

    let members: Vec<MemberItem> = client.eval_json(script).await?;

    if members.is_empty() {
        return Ok(json!([{ "index": 0, "name": "No members visible", "status": "Toggle member list first" }]));
    }
    Ok(serde_json::to_value(members).unwrap_or(json!([])))
}
