# discord-cli

基于 Rust + agent-browser 的 Discord Web CLI 本地控制面板。

## 架构

- **语言**: Rust (edition 2024)
- **浏览器自动化**: agent-browser CLI (CDP 协议)
- **目标站点**: Discord Web (https://discord.com)
- **HTTP 服务**: Axum (默认端口 12235)
- **MCP 支持**: JSON-RPC 2.0 协议
- **认证方式**: 共享密码 (Cookie / Bearer Token)

## 命令列表

| 命令 | 类别 | 说明 |
|------|------|------|
| `servers` | 读取 | 列出所有 Discord 服务器 (guilds) |
| `channels` | 读取 | 列出当前服务器的频道 (支持虚拟滚动) |
| `members` | 读取 | 列出当前频道的在线成员 |
| `read` | 读取 | 读取最近消息 (默认 20 条)，可选 `server`/`channel` 参数先切换 |
| `search` | 读取 | 搜索消息，可选 `server`/`channel` 参数先切换 |
| `send` | 写入 | 发送消息到当前频道，可选 `server`/`channel` 参数先切换 |
| `switch` | 写入 | 切换到指定服务器和/或频道 (部分匹配，不区分大小写) |
| `status` | 读取 | 检查 CDP 连接状态 |

## 前置条件

1. **agent-browser** 已安装并在 PATH 中
2. 在浏览器中打开 Discord Web: https://discord.com
3. 已在浏览器中登录 Discord

## 构建

```bash
cargo build --release
```

## 使用方式

### 1. 启动服务

```bash
./target/release/discord-cli serve
# 监听地址: http://0.0.0.0:12235
```

### 2. 设置密码 (首次运行)

```bash
curl -X POST http://localhost:12235/api/setup/password \
  -H "Content-Type: application/json" \
  -d '{"password":"your-password"}'
```

### 3. 执行命令

```bash
# 命令行方式
./target/release/discord-cli execute servers
./target/release/discord-cli execute channels
./target/release/discord-cli execute read --params '{"count":10}'

# 切换服务器和频道
./target/release/discord-cli execute switch --params '{"server":"Story","channel":"general"}'

# 切换后读取 (read/search/send 支持 server/channel 参数)
./target/release/discord-cli execute read --params '{"server":"Story","channel":"general","count":10}'

# HTTP API 方式
curl -X POST http://localhost:12235/api/execute/servers \
  -H "Authorization: Bearer your-password" \
  -H "Content-Type: application/json" \
  -d '{"params":{}}'
```

### 4. MCP 集成

服务在 `POST /mcp` 暴露以下 MCP 工具:

- `discord_servers` — 列出服务器
- `discord_channels` — 列出频道
- `discord_members` — 列出成员
- `discord_read` — 读取消息
- `discord_search` — 搜索消息
- `discord_send` — 发送消息
- `discord_switch` — 切换服务器/频道
- `discord_status` — 连接状态

## 配置

配置文件路径: `~/.config/discord-cli/config.toml`

```toml
[server]
host = "0.0.0.0"
port = 12235

[auth]
password = ""
password_changed = false

[agent_browser]
binary = "agent-browser"
cdp_url = "ws://127.0.0.1:9222"
session_name = "discord-cli"
timeout_secs = 60

[vnc]
url = ""
username = ""
password = ""
embed = true
```

## 项目结构

```
discord-cli/
├── Cargo.toml
├── build.sh
├── src/
│   ├── main.rs              # 入口
│   ├── cli.rs               # CLI 子命令
│   ├── config.rs            # 配置管理
│   ├── auth.rs              # 认证
│   ├── errors.rs            # 错误类型
│   ├── manifest.rs          # 命令/工具注册
│   ├── models.rs            # 数据模型
│   ├── response.rs          # API 响应封装
│   ├── embedded.rs          # 静态文件服务
│   ├── agent_browser/       # 浏览器绑定层
│   ├── commands/            # 命令注册 + 执行器
│   ├── discord/             # Discord 业务逻辑
│   │   ├── extract.rs       # 参数提取工具函数
│   │   └── commands/        # 8 个命令实现
│   └── server/              # HTTP 服务 + 路由
└── frontend/                # Next.js 前端控制面板
```

## 说明

- 所有命令基于 UI 自动化，无需 Discord API Token
- 浏览器必须已打开 Discord Web 并登录
- 命令共享单个浏览器会话，请按顺序调用
- DOM 选择器兼容桌面端和 Web 端 (同一 React 应用)
- `read`、`search`、`send` 命令支持可选的 `server` 和 `channel` 参数，执行前自动切换；切换失败直接报错
- `switch` 和频道切换支持虚拟滚动，会自动滚动频道列表查找目标
