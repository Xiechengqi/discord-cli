use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::agent_browser::types::AgentBrowserOptions;
use crate::commands::registry::CommandRegistry;
use crate::config::AppConfig;
use crate::discord::commands::{channels, members, read, search, send, servers, status, switch};
use crate::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct CommandExecutor {
    registry: CommandRegistry,
}

impl CommandExecutor {
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    pub async fn execute(
        &self,
        command_name: &str,
        params: Value,
        config: &AppConfig,
    ) -> AppResult<Value> {
        let command = self
            .registry
            .get(command_name)
            .ok_or_else(|| AppError::CommandNotFound(command_name.to_string()))?;

        if config.agent_browser.cdp_url.is_empty() {
            return Err(AppError::InvalidParams(
                "agent_browser.cdp_url is required".to_string(),
            ));
        }

        let client = AgentBrowserClient::new(AgentBrowserOptions {
            binary: config.agent_browser.binary.clone(),
            session_name: config.agent_browser.session_name.clone(),
            timeout_secs: config.agent_browser.timeout_secs,
        });

        match command.name {
            "channels" => channels::execute(&client, &params).await,
            "members" => members::execute(&client, &params).await,
            "read" => read::execute(&client, &params).await,
            "search" => search::execute(&client, &params).await,
            "send" => send::execute(&client, &params).await,
            "servers" => servers::execute(&client, &params).await,
            "status" => status::execute(&client, &params).await,
            "switch" => switch::execute(&client, &params).await,
            _ => Ok(json!({
                "status": "planned",
                "message": format!("Command `{}` is registered but not implemented yet", command.name),
                "params": params,
            })),
        }
    }
}
