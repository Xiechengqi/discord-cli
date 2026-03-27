use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::discord::extract::optional_string;
use crate::errors::{AppError, AppResult};

#[derive(Debug, Deserialize)]
struct SwitchResult {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ScrollInfo {
    scroll_height: f64,
    client_height: f64,
}

const SCROLL_INFO_SCRIPT: &str = r#"JSON.stringify((function() {
    var all = document.querySelectorAll('[class*=scroller]');
    var best = null;
    var bestH = 0;
    for (var i = 0; i < all.length; i++) {
        var el = all[i];
        if (el.scrollHeight > el.clientHeight && el.scrollHeight > bestH) {
            if (el.querySelector('[data-list-item-id^=channels___]')) {
                best = el;
                bestH = el.scrollHeight;
            }
        }
    }
    if (!best) { return { scroll_height: 0, client_height: 0 }; }
    best.scrollTop = 0;
    return { scroll_height: best.scrollHeight, client_height: best.clientHeight };
})())"#;

/// Switch to the given server and/or channel. Returns Err if any target is not found.
pub async fn navigate(client: &AgentBrowserClient, server: &str, channel: &str) -> AppResult<()> {
    if !server.is_empty() {
        let target = serde_json::to_string(server).unwrap_or_else(|_| "\"\"".to_string());
        let script = format!(
            r#"JSON.stringify((function(target) {{
                var target_lower = target.toLowerCase();
                var items = document.querySelectorAll('[data-list-item-id*="guildsnav___"]');
                for (var i = 0; i < items.length; i++) {{
                    var el = items[i];
                    var listId = el.getAttribute('data-list-item-id') || '';
                    if (!/guildsnav___\d{{10,}}/.test(listId)) continue;
                    var name = (el.textContent || '').trim();
                    name = name.replace(/^[\d][\d,.]*\s*[^\uff0c,]*[\uff0c,]\s*/, '');
                    name = name.trim();
                    if (name.toLowerCase().indexOf(target_lower) === -1) continue;
                    var cls = el.getAttribute('class') || '';
                    if (cls.indexOf('selected') !== -1) {{
                        return {{ status: 'already_on' }};
                    }}
                    el.click();
                    return {{ status: 'switched' }};
                }}
                return {{ status: 'not_found' }};
            }})({}))"#,
            target
        );
        let res: SwitchResult = client.eval_json(&script).await?;
        match res.status.as_str() {
            "not_found" => {
                return Err(AppError::InvalidParams(format!("server not found: {}", server)));
            }
            "switched" => {
                client.wait_ms(1000).await?;
            }
            _ => {} // already_on
        }
    }

    if !channel.is_empty() {
        let target = serde_json::to_string(channel).unwrap_or_else(|_| "\"\"".to_string());
        let try_click = format!(
            r#"JSON.stringify((function(target) {{
                var target_lower = target.toLowerCase();
                var els = document.querySelectorAll('[data-list-item-id^=channels___]');
                for (var i = 0; i < els.length; i++) {{
                    var el = els[i];
                    var label = (el.getAttribute('aria-label') || el.textContent || '').trim();
                    var commaIdx = label.search(/[,\uFF0C]/);
                    var name = commaIdx !== -1 ? label.substring(0, commaIdx).trim() : label;
                    if (name.toLowerCase().indexOf(target_lower) === -1) continue;
                    el.click();
                    return {{ status: 'switched' }};
                }}
                return {{ status: 'not_found' }};
            }})({}))"#,
            target
        );

        let res: SwitchResult = client.eval_json(&try_click).await?;
        if res.status != "switched" {
            // Scroll through channel list
            let info: ScrollInfo = client.eval_json(SCROLL_INFO_SCRIPT).await?;
            client.wait_ms(300).await?;

            let mut found = false;
            if info.scroll_height > info.client_height {
                let step = info.client_height.max(200.0);
                let mut pos = step;
                while pos < info.scroll_height + step {
                    let scroll_js = format!(
                        "(function(){{ \
                            var all = document.querySelectorAll('[class*=scroller]'); \
                            for (var i=0; i<all.length; i++) {{ \
                                if (all[i].querySelector('[data-list-item-id^=channels___]')) {{ \
                                    all[i].scrollTop = {}; break; \
                                }} \
                            }} \
                        }})()",
                        pos as u64
                    );
                    client.eval(&scroll_js).await?;
                    client.wait_ms(300).await?;

                    let res: SwitchResult = client.eval_json(&try_click).await?;
                    if res.status == "switched" {
                        found = true;
                        break;
                    }
                    pos += step;
                }
            }

            if !found {
                return Err(AppError::InvalidParams(format!(
                    "channel not found: {}",
                    channel
                )));
            }
        }
    }

    Ok(())
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let server = optional_string(params, "server").unwrap_or("").to_string();
    let channel = optional_string(params, "channel").unwrap_or("").to_string();

    if server.is_empty() && channel.is_empty() {
        return Ok(json!([{ "status": "nothing to do" }]));
    }

    // For the standalone switch command, report already_on as success too —
    // so we re-run the server check to get a friendly status message.
    let mut messages: Vec<String> = Vec::new();

    if !server.is_empty() {
        let target = serde_json::to_string(&server).unwrap_or_else(|_| "\"\"".to_string());
        let script = format!(
            r#"JSON.stringify((function(target) {{
                var target_lower = target.toLowerCase();
                var items = document.querySelectorAll('[data-list-item-id*="guildsnav___"]');
                for (var i = 0; i < items.length; i++) {{
                    var el = items[i];
                    var listId = el.getAttribute('data-list-item-id') || '';
                    if (!/guildsnav___\d{{10,}}/.test(listId)) continue;
                    var name = (el.textContent || '').trim();
                    name = name.replace(/^[\d][\d,.]*\s*[^\uff0c,]*[\uff0c,]\s*/, '');
                    name = name.trim();
                    if (name.toLowerCase().indexOf(target_lower) === -1) continue;
                    var cls = el.getAttribute('class') || '';
                    if (cls.indexOf('selected') !== -1) {{
                        return {{ status: 'already_on' }};
                    }}
                    el.click();
                    return {{ status: 'switched' }};
                }}
                return {{ status: 'not_found' }};
            }})({}))"#,
            target
        );
        let res: SwitchResult = client.eval_json(&script).await?;
        match res.status.as_str() {
            "switched" => {
                messages.push(format!("switched to server: {}", server));
                client.wait_ms(1000).await?;
            }
            "already_on" => {
                messages.push(format!("already on server: {}", server));
            }
            _ => {
                messages.push(format!("server not found: {}", server));
            }
        }
    }

    if !channel.is_empty() {
        match navigate(client, "", &channel).await {
            Ok(()) => messages.push(format!("switched to channel: {}", channel)),
            Err(e) => messages.push(e.to_string()),
        }
    }

    Ok(json!([{ "status": messages.join("; ") }]))
}
