use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::discord::extract::required_string;
use crate::errors::AppResult;

/// Navigate the browser to a URL (default: Discord)
pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    client.open(url).await?;
    Ok(json!([{ "status": format!("opened: {}", url) }]))
}
