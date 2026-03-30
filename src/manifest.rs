use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ParamSpec {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    pub name: &'static str,
    pub category: &'static str,
    pub wave: u8,
    pub execution_mode: &'static str,
    pub summary: &'static str,
    pub requires_auth: bool,
    pub params: Vec<ParamSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolSpec {
    pub name: &'static str,
    pub command: &'static str,
    pub read_only: bool,
    pub requires_auth: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillStep {
    pub r#use: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub requires_auth: bool,
    pub steps: Vec<SkillStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeSpec {
    pub binary: &'static str,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerDefaults {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModelSpec {
    pub mode: &'static str,
    pub cookie_name: &'static str,
    pub bearer_format: &'static str,
    pub first_run_requires_password_setup: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentBrowserSpec {
    pub binding: &'static str,
    pub binary_auto_detect: bool,
    pub supports_cdp_url: bool,
    pub default_session_name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescribeManifest {
    pub site: SiteSpec,
    pub runtime: RuntimeSpec,
    pub server_defaults: ServerDefaults,
    pub auth_model: AuthModelSpec,
    pub agent_browser: AgentBrowserSpec,
    pub commands: Vec<CommandSpec>,
    pub mcp_tools: Vec<ToolSpec>,
    pub skills: Vec<SkillSpec>,
}

pub fn build_manifest(config_path: String, host: String, port: u16) -> DescribeManifest {
    DescribeManifest {
        site: SiteSpec {
            id: "discord",
            name: "Discord CLI",
            version: env!("CARGO_PKG_VERSION"),
        },
        runtime: RuntimeSpec {
            binary: "discord-cli",
            config_path,
        },
        server_defaults: ServerDefaults {
            base_url: format!("http://{host}:{port}"),
            host,
            port,
        },
        auth_model: AuthModelSpec {
            mode: "shared-password",
            cookie_name: "discord_cli_token",
            bearer_format: "Authorization: Bearer <password>",
            first_run_requires_password_setup: true,
        },
        agent_browser: AgentBrowserSpec {
            binding: "cli",
            binary_auto_detect: true,
            supports_cdp_url: true,
            default_session_name: "discord-cli",
        },
        commands: command_specs(),
        mcp_tools: tool_specs(),
        skills: skill_specs(),
    }
}

pub fn command_specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            name: "servers",
            category: "read",
            wave: 1,
            execution_mode: "ui-first",
            summary: "List all Discord servers (guilds) in the sidebar",
            requires_auth: false,
            params: vec![],
        },
        CommandSpec {
            name: "channels",
            category: "read",
            wave: 1,
            execution_mode: "ui-first",
            summary: "List channels in the current Discord server",
            requires_auth: false,
            params: vec![],
        },
        CommandSpec {
            name: "open",
            category: "write",
            wave: 1,
            execution_mode: "ui-first",
            summary: "Navigate browser to a URL (default: https://discord.com)",
            requires_auth: false,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "URL to navigate to",
            }],
        },
        CommandSpec {
            name: "members",
            category: "read",
            wave: 1,
            execution_mode: "ui-first",
            summary: "List online members in the current channel",
            requires_auth: false,
            params: vec![],
        },
        CommandSpec {
            name: "read",
            category: "read",
            wave: 1,
            execution_mode: "ui-first",
            summary: "Read recent messages from the active Discord channel",
            requires_auth: false,
            params: vec![
                ParamSpec {
                    name: "server",
                    kind: "string",
                    required: false,
                    description: "Switch to this server before reading (partial match)",
                },
                ParamSpec {
                    name: "channel",
                    kind: "string",
                    required: false,
                    description: "Switch to this channel before reading (partial match)",
                },
                ParamSpec {
                    name: "count",
                    kind: "integer",
                    required: false,
                    description: "Number of messages to read (default: 20)",
                },
            ],
        },
        CommandSpec {
            name: "search",
            category: "read",
            wave: 1,
            execution_mode: "ui-first",
            summary: "Search messages in the current Discord server/channel",
            requires_auth: false,
            params: vec![
                ParamSpec {
                    name: "server",
                    kind: "string",
                    required: false,
                    description: "Switch to this server before searching (partial match)",
                },
                ParamSpec {
                    name: "channel",
                    kind: "string",
                    required: false,
                    description: "Switch to this channel before searching (partial match)",
                },
                ParamSpec {
                    name: "query",
                    kind: "string",
                    required: true,
                    description: "Search query",
                },
            ],
        },
        CommandSpec {
            name: "send",
            category: "write",
            wave: 1,
            execution_mode: "ui-first",
            summary: "Send a message to the active Discord channel",
            requires_auth: false,
            params: vec![
                ParamSpec {
                    name: "server",
                    kind: "string",
                    required: false,
                    description: "Switch to this server before sending (partial match)",
                },
                ParamSpec {
                    name: "channel",
                    kind: "string",
                    required: false,
                    description: "Switch to this channel before sending (partial match)",
                },
                ParamSpec {
                    name: "text",
                    kind: "string",
                    required: true,
                    description: "Message to send",
                },
            ],
        },
        CommandSpec {
            name: "status",
            category: "read",
            wave: 1,
            execution_mode: "ui-first",
            summary: "Check active CDP connection to Discord",
            requires_auth: false,
            params: vec![],
        },
        CommandSpec {
            name: "switch",
            category: "write",
            wave: 1,
            execution_mode: "ui-first",
            summary: "Switch to a server and/or channel by name (partial, case-insensitive)",
            requires_auth: false,
            params: vec![
                ParamSpec {
                    name: "server",
                    kind: "string",
                    required: false,
                    description: "Target server name (partial match). Omit to stay on current server.",
                },
                ParamSpec {
                    name: "channel",
                    kind: "string",
                    required: false,
                    description: "Target channel name (partial match). Omit to skip channel switch.",
                },
            ],
        },
    ]
}

pub fn tool_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "discord_servers",
            command: "servers",
            read_only: true,
            requires_auth: false,
        },
        ToolSpec {
            name: "discord_channels",
            command: "channels",
            read_only: true,
            requires_auth: false,
        },
        ToolSpec {
            name: "discord_members",
            command: "members",
            read_only: true,
            requires_auth: false,
        },
        ToolSpec {
            name: "discord_read",
            command: "read",
            read_only: true,
            requires_auth: false,
        },
        ToolSpec {
            name: "discord_search",
            command: "search",
            read_only: true,
            requires_auth: false,
        },
        ToolSpec {
            name: "discord_send",
            command: "send",
            read_only: false,
            requires_auth: false,
        },
        ToolSpec {
            name: "discord_open",
            command: "open",
            read_only: true,
            requires_auth: false,
        },
        ToolSpec {
            name: "discord_status",
            command: "status",
            read_only: true,
            requires_auth: false,
        },
        ToolSpec {
            name: "discord_switch",
            command: "switch",
            read_only: false,
            requires_auth: false,
        },
    ]
}

pub fn skill_specs() -> Vec<SkillSpec> {
    vec![
        SkillSpec {
            name: "list_workspace",
            summary: "List servers, channels, and recent messages",
            requires_auth: false,
            steps: vec![
                SkillStep { r#use: "servers" },
                SkillStep { r#use: "channels" },
                SkillStep { r#use: "read" },
            ],
        },
        SkillSpec {
            name: "search_conversation",
            summary: "Search messages in the current channel",
            requires_auth: false,
            steps: vec![SkillStep { r#use: "search" }],
        },
        SkillSpec {
            name: "send_notification",
            summary: "Send a message to the active channel",
            requires_auth: false,
            steps: vec![SkillStep { r#use: "send" }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{command_specs, skill_specs, tool_specs};

    #[test]
    fn all_seven_commands_registered() {
        let commands = command_specs();
        assert_eq!(commands.len(), 9);
        let names: Vec<&str> = commands.iter().map(|c| c.name).collect();
        assert!(names.contains(&"servers"));
        assert!(names.contains(&"channels"));
        assert!(names.contains(&"members"));
        assert!(names.contains(&"read"));
        assert!(names.contains(&"search"));
        assert!(names.contains(&"send"));
        assert!(names.contains(&"status"));
        assert!(names.contains(&"switch"));
    }

    #[test]
    fn all_seven_tools_registered() {
        let tools = tool_specs();
        assert_eq!(tools.len(), 9);
        assert!(tools.iter().any(|t| t.name == "discord_servers"));
        assert!(tools.iter().any(|t| t.name == "discord_send"));
        assert!(tools.iter().any(|t| t.name == "discord_switch"));
    }

    #[test]
    fn all_three_skills_registered() {
        let skills = skill_specs();
        assert_eq!(skills.len(), 3);
        assert!(skills.iter().any(|s| s.name == "list_workspace"));
        assert!(skills.iter().any(|s| s.name == "search_conversation"));
        assert!(skills.iter().any(|s| s.name == "send_notification"));
    }
}
