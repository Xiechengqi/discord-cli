use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;

#[derive(Debug, Deserialize, Serialize)]
struct ServerItem {
    index: u32,
    server: String,
}

pub async fn execute(client: &AgentBrowserClient, _params: &Value) -> AppResult<Value> {
    // Use *="guildsnav___" (contains) – proven to match guild nav items.
    // Filter to only items whose data-list-item-id ends with a numeric snowflake,
    // which excludes DM home ("guildsnav___home") and UI buttons.
    let script = r#"JSON.stringify((function() {
        var results = [];
        var seen = {};

        var items = document.querySelectorAll('[data-list-item-id*="guildsnav___"]');

        items.forEach(function(el) {
            var listId = el.getAttribute('data-list-item-id') || '';
            if (!/guildsnav___\d{10,}/.test(listId)) return;

            var name = (el.textContent || '').trim();
            // Strip leading mention/unread count: "30 个提及，" or "4,697 mentions, "
            name = name.replace(/^[\d][\d,.]*\s*[^\uff0c,]*[\uff0c,]\s*/, '');
            name = name.trim();
            if (!name) return;
            if (seen[name]) return;
            seen[name] = true;

            results.push({ index: results.length + 1, server: name.substring(0, 80) });
        });

        return results;
    })())"#;

    let servers: Vec<ServerItem> = client.eval_json(script).await?;

    if servers.is_empty() {
        return Ok(json!([{ "index": 0, "server": "No servers found" }]));
    }
    Ok(serde_json::to_value(servers).unwrap_or(json!([])))
}
