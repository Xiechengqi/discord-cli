use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::discord::commands::switch::navigate;
use crate::discord::extract::{optional_string, optional_u64};
use crate::errors::AppResult;

#[derive(Debug, Deserialize)]
struct ScrollInfo {
    scroll_height: f64,
    client_height: f64,
    scroll_top: f64,
}

const COLLECT_SCRIPT: &str = r#"JSON.stringify((function() {
    var results = [];
    var msgNodes = document.querySelectorAll('[class*="message"]');
    msgNodes.forEach(function(node) {
        var authorEl = node.querySelector('[class*="username"], [class*="headerText"] span, [class*="author"]');
        var timeEl = node.querySelector('time, [class*="timestamp"]');
        var contentEl = node.querySelector('[id^="message-content-"], [class*="messageContent"], [class*="content"]');
        if (contentEl) {
            results.push({
                author: authorEl ? authorEl.textContent.trim() : '—',
                time: timeEl ? (timeEl.getAttribute('datetime') || timeEl.textContent.trim()) : '',
                message: (contentEl.textContent || '').trim().substring(0, 300),
            });
        }
    });
    return results;
})())"#;

const SCROLL_INFO_SCRIPT: &str = r#"JSON.stringify((function() {
    var scrollers = document.querySelectorAll('[class*="scroller"]');
    var best = null;
    var bestH = 0;
    for (var i = 0; i < scrollers.length; i++) {
        var el = scrollers[i];
        if (el.scrollHeight > el.clientHeight && el.scrollHeight > bestH) {
            if (el.querySelector('[class*="message"]')) {
                best = el;
                bestH = el.scrollHeight;
            }
        }
    }
    if (!best) return { scroll_height: 0, client_height: 0, scroll_top: 0 };
    return { scroll_height: best.scrollHeight, client_height: best.clientHeight, scroll_top: best.scrollTop };
})())"#;

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let server = optional_string(params, "server").unwrap_or("").to_string();
    let channel = optional_string(params, "channel").unwrap_or("").to_string();
    navigate(client, &server, &channel).await?;

    let count = optional_u64(params, "count", 20);

    // Find scroller and get dimensions
    let info: ScrollInfo = client.eval_json(SCROLL_INFO_SCRIPT).await?;
    client.wait_ms(300).await?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut ordered: Vec<(String, String, String)> = Vec::new();

    // Collect at current position
    {
        let batch: Vec<serde_json::Map<String, Value>> = client.eval_json(COLLECT_SCRIPT).await?;
        for item in batch {
            let author = item.get("author").and_then(Value::as_str).unwrap_or("").to_string();
            let time = item.get("time").and_then(Value::as_str).unwrap_or("").to_string();
            let msg = item.get("message").and_then(Value::as_str).unwrap_or("").to_string();
            if msg.is_empty() { continue; }
            let key = format!("{}|{}|{}", author, time, msg);
            if seen.insert(key) {
                ordered.push((author, time, msg));
            }
        }
    }

    // Scroll to bottom first
    if info.scroll_height > 0.0 {
        let scroll_bottom = r#"(function() {
            var scrollers = document.querySelectorAll('[class*="scroller"]');
            for (var i = 0; i < scrollers.length; i++) {
                var el = scrollers[i];
                if (el.scrollHeight > el.clientHeight) {
                    if (el.querySelector('[class*="message"]')) {
                        el.scrollTop = el.scrollHeight;
                        return;
                    }
                }
            }
        })()"#;
        client.eval(scroll_bottom).await?;
        client.wait_ms(500).await?;
    }

    // Scroll up step by step to collect all messages
    if info.scroll_height > info.client_height && ordered.len() < count as usize * 2 {
        let step = info.client_height.max(300.0);
        let mut pos = info.scroll_height;

        while pos > 0.0 && ordered.len() < count as usize * 3 {
            let scroll_js = format!(
                r#"(function() {{
                    var scrollers = document.querySelectorAll('[class*="scroller"]');
                    for (var i = 0; i < scrollers.length; i++) {{
                        var el = scrollers[i];
                        if (el.querySelector('[class*="message"]')) {{
                            el.scrollTop = {};
                            return;
                        }}
                    }}
                }})()"#,
                pos as u64
            );
            client.eval(&scroll_js).await?;
            client.wait_ms(400).await?;

            let batch: Vec<serde_json::Map<String, Value>> = client.eval_json(COLLECT_SCRIPT).await?;
            for item in batch {
                let author = item.get("author").and_then(Value::as_str).unwrap_or("").to_string();
                let time = item.get("time").and_then(Value::as_str).unwrap_or("").to_string();
                let msg = item.get("message").and_then(Value::as_str).unwrap_or("").to_string();
                if msg.is_empty() { continue; }
                let key = format!("{}|{}|{}", author, time, msg);
                if seen.insert(key) {
                    ordered.push((author, time, msg));
                }
            }

            if pos <= step {
                break;
            }
            pos -= step;
        }
    }

    // Return up to count messages (newest first)
    let result: Vec<Value> = ordered
        .into_iter()
        .rev()
        .take(count as usize)
        .map(|(author, time, message)| json!({ "author": author, "time": time, "message": message }))
        .collect();

    if result.is_empty() {
        return Ok(json!([{ "author": "System", "time": "", "message": "No messages found in the current channel." }]));
    }

    Ok(Value::Array(result))
}
