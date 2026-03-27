use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::discord::commands::switch::navigate;
use crate::discord::extract::{optional_string, required_string};
use crate::errors::AppResult;

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let server = optional_string(params, "server").unwrap_or("").to_string();
    let channel = optional_string(params, "channel").unwrap_or("").to_string();
    navigate(client, &server, &channel).await?;

    let text = required_string(params, "text")?;

    // Focus the Slate editor and insert text
    let insert_script = format!(
        r#"(function(text) {{
            var editor = document.querySelector('[role="textbox"][data-slate-editor="true"], [class*="slateTextArea"]');
            if (!editor) throw new Error('Could not find Discord message input. Make sure a channel is open.');

            editor.focus();
            document.execCommand('insertText', false, text);
        }})({})"#,
        serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
    );
    client.eval(&insert_script).await?;

    client.wait_ms(300).await?;

    // Press Enter to send
    client
        .eval(
            r#"(function() {
            var editor = document.querySelector('[role="textbox"][data-slate-editor="true"], [class*="slateTextArea"]');
            if (editor) {
                editor.dispatchEvent(new KeyboardEvent('keydown', {
                    key: 'Enter', keyCode: 13, which: 13,
                    bubbles: true, cancelable: true
                }));
            }
        })()"#,
        )
        .await?;

    Ok(json!([{ "status": "Success" }]))
}
