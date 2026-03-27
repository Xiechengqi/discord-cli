use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::discord::commands::switch::navigate;
use crate::discord::extract::{optional_string, required_string};
use crate::errors::AppResult;

#[derive(Debug, Deserialize, Serialize)]
struct SearchResultItem {
    index: u32,
    author: String,
    message: String,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let server = optional_string(params, "server").unwrap_or("").to_string();
    let channel = optional_string(params, "channel").unwrap_or("").to_string();
    navigate(client, &server, &channel).await?;

    let query = required_string(params, "query")?;

    // Open search with Ctrl+F (Discord Web runs in a regular browser)
    client
        .eval(
            r#"(function() {
            document.dispatchEvent(new KeyboardEvent('keydown', {
                key: 'f', code: 'KeyF', keyCode: 70, which: 70,
                ctrlKey: true, metaKey: false,
                bubbles: true, cancelable: true
            }));
        })()"#,
        )
        .await?;
    client.wait_ms(500).await?;

    // Type query into search box
    let type_script = format!(
        r#"(function(q) {{
            var input = document.querySelector('[aria-label*="Search"], [class*="searchBar"] input, [placeholder*="Search"]');
            if (!input) throw new Error('Search input not found');
            input.focus();
            var setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
            setter.call(input, q);
            input.dispatchEvent(new Event('input', {{ bubbles: true }}));
        }})({})"#,
        serde_json::to_string(query).unwrap_or_else(|_| "\"\"".to_string())
    );
    client.eval(&type_script).await?;

    // Press Enter to trigger search
    client
        .eval(
            r#"(function() {
            var input = document.querySelector('[aria-label*="Search"], [class*="searchBar"] input, [placeholder*="Search"]');
            if (input) {
                input.dispatchEvent(new KeyboardEvent('keydown', {
                    key: 'Enter', keyCode: 13, which: 13,
                    bubbles: true, cancelable: true
                }));
            }
        })()"#,
        )
        .await?;
    client.wait_ms(2000).await?;

    // Scrape search results
    let scrape_script = r#"JSON.stringify((function() {
        var items = [];
        var resultNodes = document.querySelectorAll('[class*="searchResult_"], [id*="search-result"]');

        resultNodes.forEach(function(node, i) {
            var author = (node.querySelector('[class*="username"]') || {}).textContent || '—';
            var contentEl = node.querySelector('[id^="message-content-"], [class*="messageContent"]');
            var content = contentEl ? contentEl.textContent.trim() : (node.textContent || '').trim();
            items.push({
                index: i + 1,
                author: author.trim(),
                message: (content || '').substring(0, 200),
            });
        });

        return items;
    })())"#;

    let results: Vec<SearchResultItem> = client.eval_json(scrape_script).await?;

    // Close search
    client
        .eval(
            r#"(function() {
            document.dispatchEvent(new KeyboardEvent('keydown', {
                key: 'Escape', keyCode: 27, which: 27,
                bubbles: true, cancelable: true
            }));
        })()"#,
        )
        .await?;

    if results.is_empty() {
        return Ok(
            json!([{ "index": 0, "author": "System", "message": format!("No results for \"{}\"", query) }]),
        );
    }
    Ok(serde_json::to_value(results).unwrap_or(json!([])))
}
