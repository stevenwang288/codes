use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use code_common::CliConfigOverrides;
use code_core::config::Config;
use code_core::config::ConfigOverrides;
use code_core::config::find_code_home;
use code_core::config::load_global_mcp_servers;
use code_core::config::write_global_mcp_servers;
use code_core::config_types::McpServerConfig;
use code_core::config_types::McpServerTransportConfig;
use code_i18n;

/// Subcommands:
/// - `serve`  — run the MCP server on stdio
/// - `list`   — list configured servers (with `--json`)
/// - `get`    — show a single server (with `--json`)
/// - `add`    — add a server launcher entry to `~/.code/config.toml` (Code also reads legacy `~/.codex/config.toml`)
/// - `remove` — delete a server entry
#[derive(Debug, clap::Parser)]
pub struct McpCli {
    #[clap(flatten)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    pub subcommand: McpSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum McpSubcommand {
    List(ListArgs),

    Get(GetArgs),

    Add(AddArgs),

    Remove(RemoveArgs),
}

#[derive(Debug, clap::Parser)]
pub struct ListArgs {
    /// Output the configured servers as JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct GetArgs {
    /// Name of the MCP server to display.
    pub name: String,

    /// Output the server configuration as JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct AddArgs {
    /// Name for the MCP server configuration.
    pub name: String,

    /// URL of a remote MCP server.
    ///
    /// When `--bearer-token` is omitted, Code records the server as a stdio
    /// launcher using `npx -y mcp-remote <url>` so the MCP server can handle
    /// OAuth flows.
    #[arg(long)]
    pub url: Option<String>,

    /// Optional bearer token to use with `--url` for static authentication.
    ///
    /// When set, Code records the server as a `streamable_http` MCP server.
    #[arg(long)]
    pub bearer_token: Option<String>,

    /// Environment variables to set when launching the server.
    #[arg(long, value_parser = parse_env_pair, value_name = "KEY=VALUE")]
    pub env: Vec<(String, String)>,

    /// Command to launch the MCP server.
    #[arg(trailing_var_arg = true, num_args = 0..)]
    pub command: Vec<String>,
}

#[derive(Debug, clap::Parser)]
pub struct RemoveArgs {
    /// Name of the MCP server configuration to remove.
    pub name: String,
}

impl McpCli {
    pub async fn run(self) -> Result<()> {
        let McpCli {
            config_overrides,
            subcommand,
        } = self;

        match subcommand {
            McpSubcommand::List(args) => {
                run_list(&config_overrides, args)?;
            }
            McpSubcommand::Get(args) => {
                run_get(&config_overrides, args)?;
            }
            McpSubcommand::Add(args) => {
                run_add(&config_overrides, args)?;
            }
            McpSubcommand::Remove(args) => {
                run_remove(&config_overrides, args)?;
            }
        }

        Ok(())
    }
}

fn build_mcp_transport_for_add(
    url: Option<String>,
    bearer_token: Option<String>,
    env: Option<HashMap<String, String>>,
    command: Vec<String>,
) -> Result<McpServerTransportConfig> {
    if let Some(url) = url {
        if !command.is_empty() {
            bail!("{}", code_i18n::tr_plain("cli.mcp.add.url_with_command_error"));
        }
        if let Some(bearer_token) = bearer_token {
            return Ok(McpServerTransportConfig::StreamableHttp {
                url,
                bearer_token: Some(bearer_token),
            });
        }
        return Ok(McpServerTransportConfig::Stdio {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-remote".to_string(), url],
            env,
        });
    }

    if bearer_token.is_some() {
        bail!("{}", code_i18n::tr_plain("cli.mcp.add.bearer_requires_url_error"));
    }

    let mut command_parts = command.into_iter();
    let command_bin = command_parts
        .next()
        .ok_or_else(|| anyhow!(code_i18n::tr_plain("cli.mcp.add.command_required_error")))?;
    let command_args: Vec<String> = command_parts.collect();
    Ok(McpServerTransportConfig::Stdio {
        command: command_bin,
        args: command_args,
        env,
    })
}

fn run_add(config_overrides: &CliConfigOverrides, add_args: AddArgs) -> Result<()> {
    // Validate any provided overrides even though they are not currently applied.
    config_overrides.parse_overrides().map_err(|e| anyhow!(e))?;

    let AddArgs {
        name,
        url,
        bearer_token,
        env,
        command,
    } = add_args;

    validate_server_name(&name)?;

    let env_map = if env.is_empty() {
        None
    } else {
        let mut map = HashMap::new();
        for (key, value) in env {
            map.insert(key, value);
        }
        Some(map)
    };

    let code_home = find_code_home().context(code_i18n::tr_plain("cli.config.find_code_home_failed"))?;
    let mut servers = load_global_mcp_servers(&code_home)
        .with_context(|| format!("failed to load MCP servers from {}", code_home.display()))?;

    let transport = build_mcp_transport_for_add(url, bearer_token, env_map, command)?;

    let new_entry = McpServerConfig {
        transport,
        startup_timeout_sec: None,
        tool_timeout_sec: None,
    };

    servers.insert(name.clone(), new_entry);

    write_global_mcp_servers(&code_home, &servers)
        .with_context(|| format!("failed to write MCP servers to {}", code_home.display()))?;

    let ui_language = code_i18n::current_language();
    println!(
        "{}",
        code_i18n::tr_args(ui_language, "cli.mcp.added", &[("name", name.as_str())])
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_with_url_defaults_to_mcp_remote() {
        let transport = build_mcp_transport_for_add(
            Some("https://mcp.example.com/mcp".to_string()),
            None,
            None,
            Vec::new(),
        )
        .expect("transport");

        match transport {
            McpServerTransportConfig::Stdio { command, args, env } => {
                assert_eq!(command, "npx");
                assert_eq!(args[0], "-y");
                assert_eq!(args[1], "mcp-remote");
                assert_eq!(args[2], "https://mcp.example.com/mcp");
                assert!(env.is_none());
            }
            _ => panic!("expected stdio transport"),
        }
    }

    #[test]
    fn add_with_url_and_bearer_token_uses_streamable_http() {
        let transport = build_mcp_transport_for_add(
            Some("https://mcp.example.com/mcp".to_string()),
            Some("token".to_string()),
            None,
            Vec::new(),
        )
        .expect("transport");

        match transport {
            McpServerTransportConfig::StreamableHttp { url, bearer_token } => {
                assert_eq!(url, "https://mcp.example.com/mcp");
                assert_eq!(bearer_token.as_deref(), Some("token"));
            }
            _ => panic!("expected streamable http transport"),
        }
    }
}

fn run_remove(config_overrides: &CliConfigOverrides, remove_args: RemoveArgs) -> Result<()> {
    config_overrides.parse_overrides().map_err(|e| anyhow!(e))?;

    let RemoveArgs { name } = remove_args;

    validate_server_name(&name)?;

    let code_home = find_code_home().context(code_i18n::tr_plain("cli.config.find_code_home_failed"))?;
    let mut servers = load_global_mcp_servers(&code_home)
        .with_context(|| format!("failed to load MCP servers from {}", code_home.display()))?;

    let removed = servers.remove(&name).is_some();

    if removed {
        write_global_mcp_servers(&code_home, &servers)
            .with_context(|| format!("failed to write MCP servers to {}", code_home.display()))?;
    }

    let ui_language = code_i18n::current_language();
    if removed {
        println!(
            "{}",
            code_i18n::tr_args(ui_language, "cli.mcp.removed", &[("name", name.as_str())])
        );
    } else {
        println!(
            "{}",
            code_i18n::tr_args(ui_language, "cli.mcp.not_found", &[("name", name.as_str())])
        );
    }

    Ok(())
}

fn run_list(config_overrides: &CliConfigOverrides, list_args: ListArgs) -> Result<()> {
    let overrides = config_overrides.parse_overrides().map_err(|e| anyhow!(e))?;
    let config = Config::load_with_cli_overrides(overrides, ConfigOverrides::default())
        .context(code_i18n::tr_plain("cli.config.load_failed"))?;

    let mut entries: Vec<_> = config.mcp_servers.iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    if list_args.json {
        let json_entries: Vec<_> = entries
            .into_iter()
            .map(|(name, cfg)| {
                let transport = match &cfg.transport {
                    McpServerTransportConfig::Stdio { command, args, env } => serde_json::json!({
                        "type": "stdio",
                        "command": command,
                        "args": args,
                        "env": env,
                    }),
                    McpServerTransportConfig::StreamableHttp { url, bearer_token } => {
                        serde_json::json!({
                            "type": "streamable_http",
                            "url": url,
                            "bearer_token": bearer_token,
                        })
                    }
                };

                serde_json::json!({
                    "name": name,
                    "transport": transport,
                    "startup_timeout_sec": cfg.startup_timeout_sec.map(|d| d.as_secs_f64()),
                    "tool_timeout_sec": cfg.tool_timeout_sec.map(|d| d.as_secs_f64()),
                })
            })
            .collect();
        let output = serde_json::to_string_pretty(&json_entries)?;
        println!("{output}");
        return Ok(());
    }

    if entries.is_empty() {
        println!("{}", code_i18n::tr_plain("cli.mcp.none_configured"));
        return Ok(());
    }

    let mut stdio_rows: Vec<[String; 4]> = Vec::new();
    let mut http_rows: Vec<[String; 3]> = Vec::new();

    for (name, cfg) in entries {
        match &cfg.transport {
            McpServerTransportConfig::Stdio { command, args, env } => {
                let args_display = if args.is_empty() {
                    "-".to_string()
                } else {
                    args.join(" ")
                };
                let env_display = match env.as_ref() {
                    None => "-".to_string(),
                    Some(map) if map.is_empty() => "-".to_string(),
                    Some(map) => {
                        let mut pairs: Vec<_> = map.iter().collect();
                        pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
                        pairs
                            .into_iter()
                            .map(|(k, v)| format!("{k}={v}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                };
                stdio_rows.push([name.clone(), command.clone(), args_display, env_display]);
            }
            McpServerTransportConfig::StreamableHttp { url, bearer_token } => {
                let has_bearer = if bearer_token.is_some() {
                    "True"
                } else {
                    "False"
                };
                http_rows.push([name.clone(), url.clone(), has_bearer.into()]);
            }
        }
    }

    if !stdio_rows.is_empty() {
        let headers = [
            code_i18n::tr_plain("cli.mcp.table.name"),
            code_i18n::tr_plain("cli.mcp.table.command"),
            code_i18n::tr_plain("cli.mcp.table.args"),
            code_i18n::tr_plain("cli.mcp.table.env"),
        ];
        let mut widths = [headers[0].len(), headers[1].len(), headers[2].len(), headers[3].len()];
        for row in &stdio_rows {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(cell.len());
            }
        }

        println!(
            "{:<name_w$}  {:<cmd_w$}  {:<args_w$}  {:<env_w$}",
            headers[0],
            headers[1],
            headers[2],
            headers[3],
            name_w = widths[0],
            cmd_w = widths[1],
            args_w = widths[2],
            env_w = widths[3],
        );

        for row in &stdio_rows {
            println!(
                "{:<name_w$}  {:<cmd_w$}  {:<args_w$}  {:<env_w$}",
                row[0],
                row[1],
                row[2],
                row[3],
                name_w = widths[0],
                cmd_w = widths[1],
                args_w = widths[2],
                env_w = widths[3],
            );
        }
    }

    if !stdio_rows.is_empty() && !http_rows.is_empty() {
        println!();
    }

    if !http_rows.is_empty() {
        let headers = [
            code_i18n::tr_plain("cli.mcp.table.name"),
            code_i18n::tr_plain("cli.mcp.table.url"),
            code_i18n::tr_plain("cli.mcp.table.has_bearer_token"),
        ];
        let mut widths = [headers[0].len(), headers[1].len(), headers[2].len()];
        for row in &http_rows {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(cell.len());
            }
        }

        println!(
            "{:<name_w$}  {:<url_w$}  {:<token_w$}",
            headers[0],
            headers[1],
            headers[2],
            name_w = widths[0],
            url_w = widths[1],
            token_w = widths[2],
        );

        for row in &http_rows {
            println!(
                "{:<name_w$}  {:<url_w$}  {:<token_w$}",
                row[0],
                row[1],
                row[2],
                name_w = widths[0],
                url_w = widths[1],
                token_w = widths[2],
            );
        }
    }

    Ok(())
}

fn run_get(config_overrides: &CliConfigOverrides, get_args: GetArgs) -> Result<()> {
    let overrides = config_overrides.parse_overrides().map_err(|e| anyhow!(e))?;
    let config = Config::load_with_cli_overrides(overrides, ConfigOverrides::default())
        .context(code_i18n::tr_plain("cli.config.load_failed"))?;

    let Some(server) = config.mcp_servers.get(&get_args.name) else {
        let ui_language = code_i18n::current_language();
        bail!(
            "{}",
            code_i18n::tr_args(ui_language, "cli.mcp.not_found", &[("name", get_args.name.as_str())])
        );
    };

    if get_args.json {
        let transport = match &server.transport {
            McpServerTransportConfig::Stdio { command, args, env } => serde_json::json!({
                "type": "stdio",
                "command": command,
                "args": args,
                "env": env,
            }),
            McpServerTransportConfig::StreamableHttp { url, bearer_token } => serde_json::json!({
                "type": "streamable_http",
                "url": url,
                "bearer_token": bearer_token,
            }),
        };
        let output = serde_json::to_string_pretty(&serde_json::json!({
            "name": get_args.name,
            "transport": transport,
            "startup_timeout_sec": server.startup_timeout_sec.map(|d| d.as_secs_f64()),
            "tool_timeout_sec": server.tool_timeout_sec.map(|d| d.as_secs_f64()),
        }))?;
        println!("{output}");
        return Ok(());
    }

    println!("{}", get_args.name);
    let ui_language = code_i18n::current_language();
    match &server.transport {
        McpServerTransportConfig::Stdio { command, args, env } => {
            println!(
                "  {}",
                code_i18n::tr_args(ui_language, "cli.mcp.get.transport", &[("value", "stdio")])
            );
            println!(
                "  {}",
                code_i18n::tr_args(ui_language, "cli.mcp.get.command", &[("value", command.as_str())])
            );
            let args_display = if args.is_empty() {
                "-".to_string()
            } else {
                args.join(" ")
            };
            println!(
                "  {}",
                code_i18n::tr_args(ui_language, "cli.mcp.get.args", &[("value", &args_display)])
            );
            let env_display = match env.as_ref() {
                None => "-".to_string(),
                Some(map) if map.is_empty() => "-".to_string(),
                Some(map) => {
                    let mut pairs: Vec<_> = map.iter().collect();
                    pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
                    pairs
                        .into_iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            };
            println!(
                "  {}",
                code_i18n::tr_args(ui_language, "cli.mcp.get.env", &[("value", &env_display)])
            );
        }
        McpServerTransportConfig::StreamableHttp { url, bearer_token } => {
            println!(
                "  {}",
                code_i18n::tr_args(
                    ui_language,
                    "cli.mcp.get.transport",
                    &[("value", "streamable_http")]
                )
            );
            println!(
                "  {}",
                code_i18n::tr_args(ui_language, "cli.mcp.get.url", &[("value", url.as_str())])
            );
            let token_display = bearer_token
                .as_ref()
                .map(|_| "<redacted>".to_string())
                .unwrap_or_else(|| "-".to_string());
            println!(
                "  {}",
                code_i18n::tr_args(
                    ui_language,
                    "cli.mcp.get.bearer_token",
                    &[("value", &token_display)]
                )
            );
        }
    }
    if let Some(timeout) = server.startup_timeout_sec {
        println!(
            "  {}",
            code_i18n::tr_args(
                ui_language,
                "cli.mcp.get.startup_timeout_sec",
                &[("value", &format!("{:.3}", timeout.as_secs_f64()))]
            )
        );
    }
    if let Some(timeout) = server.tool_timeout_sec {
        println!(
            "  {}",
            code_i18n::tr_args(
                ui_language,
                "cli.mcp.get.tool_timeout_sec",
                &[("value", &format!("{:.3}", timeout.as_secs_f64()))]
            )
        );
    }
    println!(
        "  {}",
        code_i18n::tr_args(ui_language, "cli.mcp.remove_hint", &[("name", get_args.name.as_str())])
    );

    Ok(())
}

fn parse_env_pair(raw: &str) -> Result<(String, String), String> {
    let mut parts = raw.splitn(2, '=');
    let key = parts
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| code_i18n::tr_plain("cli.mcp.env_pair_invalid").to_string())?;
    let value = parts
        .next()
        .map(str::to_string)
        .ok_or_else(|| code_i18n::tr_plain("cli.mcp.env_pair_invalid").to_string())?;

    Ok((key.to_string(), value))
}

fn validate_server_name(name: &str) -> Result<()> {
    let is_valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if is_valid {
        Ok(())
    } else {
        let ui_language = code_i18n::current_language();
        bail!(
            "{}",
            code_i18n::tr_args(
                ui_language,
                "cli.mcp.invalid_server_name",
                &[("name", name)]
            )
        );
    }
}
