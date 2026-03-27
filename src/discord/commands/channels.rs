use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;

#[derive(Debug, Deserialize)]
struct ScrollInfo {
    scroll_height: f64,
    client_height: f64,
}

// No double-quotes inside the JS – attribute selector values are unquoted
// (valid CSS when value contains only [a-zA-Z0-9_-])
const COLLECT_SCRIPT: &str = r#"JSON.stringify((function() {
    var results = [];
    var els = document.querySelectorAll('[data-list-item-id^=channels___]');
    if (els.length === 0) {
        els = document.querySelectorAll('a[href*=/channels/]');
    }
    els.forEach(function(el) {
        var label = (el.getAttribute('aria-label') || el.textContent || '').trim();
        if (!label) return;
        if (/\bcategory\b/i.test(label)) return;

        var name = label;
        var type = 'Text';

        var commaM = label.match(/^(.+?)[,\uFF0C]\s*(.+)$/);
        if (commaM) {
            name = commaM[1].trim();
            var rest = commaM[2].toLowerCase();
            if (rest.indexOf('voice') !== -1) type = 'Voice';
            else if (rest.indexOf('forum') !== -1) type = 'Forum';
            else if (rest.indexOf('announcement') !== -1) type = 'Announcement';
            else if (rest.indexOf('stage') !== -1) type = 'Stage';
        } else {
            var parenM = label.match(/^(.+?)\s*[\uFF08(](.+?)[\uFF09)]\s*$/);
            if (parenM) {
                name = parenM[1].trim();
                var rawType = parenM[2].toLowerCase();
                if (rawType.indexOf('voice') !== -1) type = 'Voice';
                else if (rawType.indexOf('forum') !== -1) type = 'Forum';
                else if (rawType.indexOf('announcement') !== -1) type = 'Announcement';
                else if (rawType.indexOf('stage') !== -1) type = 'Stage';
            }
        }

        if (!name) return;
        results.push({ channel: name.substring(0, 80), type: type });
    });
    return results;
})())"#;

// Returns the scroller container to top and reports its dimensions
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

pub async fn execute(client: &AgentBrowserClient, _params: &Value) -> AppResult<Value> {
    let info: ScrollInfo = client.eval_json(SCROLL_INFO_SCRIPT).await?;
    client.wait_ms(300).await?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut ordered: Vec<(String, String)> = Vec::new();

    let mut collect = |batch: Vec<serde_json::Map<String, Value>>| {
        for item in batch {
            let name = item.get("channel").and_then(Value::as_str).unwrap_or("").to_string();
            let typ = item.get("type").and_then(Value::as_str).unwrap_or("Text").to_string();
            if !name.is_empty() && seen.insert(name.clone()) {
                ordered.push((name, typ));
            }
        }
    };

    let batch: Vec<serde_json::Map<String, Value>> = client.eval_json(COLLECT_SCRIPT).await?;
    collect(batch);

    if info.scroll_height > info.client_height {
        let step = info.client_height.max(200.0);
        let mut pos = step;
        while pos < info.scroll_height + step {
            // No double-quotes: [data-list-item-id^=channels___] without value quotes
            let scroll_js = format!(
                "(function(){{ \
                    var all = document.querySelectorAll('[class*=scroller]'); \
                    var c = null; \
                    for (var i=0; i<all.length; i++) {{ \
                        if (all[i].querySelector('[data-list-item-id^=channels___]')) {{ c = all[i]; break; }} \
                    }} \
                    if (c) c.scrollTop = {}; \
                }})()",
                pos as u64
            );
            client.eval(&scroll_js).await?;
            client.wait_ms(300).await?;

            let batch: Vec<serde_json::Map<String, Value>> = client.eval_json(COLLECT_SCRIPT).await?;
            collect(batch);

            pos += step;
        }
    }

    if ordered.is_empty() {
        return Ok(json!([{ "index": 0, "channel": "No channels found", "type": "—" }]));
    }

    let result: Vec<Value> = ordered
        .into_iter()
        .enumerate()
        .map(|(i, (name, typ))| json!({ "index": i + 1, "channel": name, "type": typ }))
        .collect();

    Ok(Value::Array(result))
}
