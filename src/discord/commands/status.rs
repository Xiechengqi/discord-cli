use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;

pub async fn execute(client: &AgentBrowserClient, _params: &Value) -> AppResult<Value> {
    let url_result = client.eval("window.location.href").await?;
    let title_result = client.eval("document.title").await?;

    let url = url_result.result.as_str().unwrap_or("unknown").to_string();
    let title = title_result.result.as_str().unwrap_or("unknown").to_string();

    Ok(json!([{
        "status": "Connected",
        "url": url,
        "title": title
    }]))
}
