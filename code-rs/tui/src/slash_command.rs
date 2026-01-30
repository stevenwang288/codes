use std::sync::OnceLock;

use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

const BUILD_PROFILE: Option<&str> = option_env!("CODES_PROFILE");

fn demo_command_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        let profile_matches = |
            profile: &str
        | {
            let normalized = profile.trim().to_ascii_lowercase();
            normalized == "perf" || normalized.starts_with("dev")
        };

        if let Some(profile) = BUILD_PROFILE.or(option_env!("PROFILE")) {
            if profile_matches(profile) {
                return true;
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            let path = exe_path.to_string_lossy().to_ascii_lowercase();
            if path.contains("target/dev-fast/")
                || path.contains("target/dev/")
                || path.contains("target/perf/")
            {
                return true;
            }
        }

        cfg!(debug_assertions)
    })
}

/// Commands that can be invoked by starting a message with a leading slash.
///
/// IMPORTANT: When adding or changing slash commands, also update
/// `docs/slash-commands.md` at the repo root so users can discover them easily.
/// This enum is the source of truth for the list and ordering shown in the UI.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Browser,
    Chrome,
    New,
    Init,
    Compact,
    Undo,
    Review,
    Cloud,
    Diff,
    Mention,
    Cmd,
    Status,
    Limits,
    #[strum(serialize = "update", serialize = "upgrade")]
    Update,
    Notifications,
    Theme,
    Settings,
    #[strum(serialize = "l")]
    Lang,
    Model,
    Reasoning,
    Verbosity,
    Prompts,
    Skills,
    Perf,
    Demo,
    Agents,
    Auto,
    Branch,
    Merge,
    Push,
    Validation,
    Mcp,
    Resume,
    Login,
    // Prompt-expanding commands
    Plan,
    Solve,
    Code,
    Logout,
    Quit,
    #[cfg(debug_assertions)]
    TestApproval,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::Chrome => code_i18n::tr_plain("tui.slash.desc.chrome"),
            SlashCommand::Browser => code_i18n::tr_plain("tui.slash.desc.browser"),
            SlashCommand::Resume => code_i18n::tr_plain("tui.slash.desc.resume"),
            SlashCommand::Plan => code_i18n::tr_plain("tui.slash.desc.plan"),
            SlashCommand::Solve => code_i18n::tr_plain("tui.slash.desc.solve"),
            SlashCommand::Code => code_i18n::tr_plain("tui.slash.desc.code"),
            SlashCommand::Reasoning => code_i18n::tr_plain("tui.slash.desc.reasoning"),
            SlashCommand::Verbosity => code_i18n::tr_plain("tui.slash.desc.verbosity"),
            SlashCommand::New => code_i18n::tr_plain("tui.slash.desc.new"),
            SlashCommand::Init => code_i18n::tr_plain("tui.slash.desc.init"),
            SlashCommand::Compact => code_i18n::tr_plain("tui.slash.desc.compact"),
            SlashCommand::Undo => code_i18n::tr_plain("tui.slash.desc.undo"),
            SlashCommand::Review => code_i18n::tr_plain("tui.slash.desc.review"),
            SlashCommand::Cloud => code_i18n::tr_plain("tui.slash.desc.cloud"),
            SlashCommand::Quit => code_i18n::tr_plain("tui.slash.desc.quit"),
            SlashCommand::Diff => code_i18n::tr_plain("tui.slash.desc.diff"),
            SlashCommand::Mention => code_i18n::tr_plain("tui.slash.desc.mention"),
            SlashCommand::Cmd => code_i18n::tr_plain("tui.slash.desc.cmd"),
            SlashCommand::Status => code_i18n::tr_plain("tui.slash.desc.status"),
            SlashCommand::Limits => code_i18n::tr_plain("tui.slash.desc.limits"),
            SlashCommand::Update => code_i18n::tr_plain("tui.slash.desc.update"),
            SlashCommand::Notifications => code_i18n::tr_plain("tui.slash.desc.notifications"),
            SlashCommand::Theme => code_i18n::tr_plain("tui.slash.desc.theme"),
            SlashCommand::Settings => code_i18n::tr_plain("tui.slash.desc.settings"),
            SlashCommand::Lang => code_i18n::tr_plain("tui.slash.desc.lang"),
            SlashCommand::Prompts => code_i18n::tr_plain("tui.slash.desc.prompts"),
            SlashCommand::Skills => code_i18n::tr_plain("tui.slash.desc.skills"),
            SlashCommand::Model => code_i18n::tr_plain("tui.slash.desc.model"),
            SlashCommand::Agents => code_i18n::tr_plain("tui.slash.desc.agents"),
            SlashCommand::Auto => code_i18n::tr_plain("tui.slash.desc.auto"),
            SlashCommand::Branch => code_i18n::tr_plain("tui.slash.desc.branch"),
            SlashCommand::Merge => code_i18n::tr_plain("tui.slash.desc.merge"),
            SlashCommand::Push => code_i18n::tr_plain("tui.slash.desc.push"),
            SlashCommand::Validation => code_i18n::tr_plain("tui.slash.desc.validation"),
            SlashCommand::Mcp => code_i18n::tr_plain("tui.slash.desc.mcp"),
            SlashCommand::Perf => code_i18n::tr_plain("tui.slash.desc.perf"),
            SlashCommand::Demo => code_i18n::tr_plain("tui.slash.desc.demo"),
            SlashCommand::Login => code_i18n::tr_plain("tui.slash.desc.login"),
            SlashCommand::Logout => code_i18n::tr_plain("tui.slash.desc.logout"),
            #[cfg(debug_assertions)]
            SlashCommand::TestApproval => code_i18n::tr_plain("tui.slash.desc.test_approval"),
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Returns true if this command should expand into a prompt for the LLM.
    pub fn is_prompt_expanding(self) -> bool {
        matches!(
            self,
            SlashCommand::Plan | SlashCommand::Solve | SlashCommand::Code
        )
    }

    pub fn settings_section_from_args<'a>(&self, args: &'a str) -> Option<&'a str> {
        if *self != SlashCommand::Settings {
            return None;
        }
        let section = args.split_whitespace().next().unwrap_or("");
        if section.is_empty() {
            None
        } else {
            Some(section)
        }
    }

    /// Returns true if this command requires additional arguments after the command.
    pub fn requires_arguments(self) -> bool {
        matches!(
            self,
            SlashCommand::Plan | SlashCommand::Solve | SlashCommand::Code
        )
    }

    pub fn is_available(self) -> bool {
        match self {
            SlashCommand::Demo => demo_command_enabled(),
            _ => true,
        }
    }

    /// Expands a prompt-expanding command into a full prompt for the LLM.
    /// Returns None if the command is not a prompt-expanding command.
    pub fn expand_prompt(self, args: &str) -> Option<String> {
        if !self.is_prompt_expanding() {
            return None;
        }

        // Use the slash_commands module from core to generate the prompts
        // Note: We pass None for agents here as the TUI doesn't have access to the session config
        // The actual agents will be determined when the agent tool is invoked
        match self {
            SlashCommand::Plan => Some(code_core::slash_commands::format_plan_command(
                args, None, None,
            )),
            SlashCommand::Solve => Some(code_core::slash_commands::format_solve_command(
                args, None, None,
            )),
            SlashCommand::Code => Some(code_core::slash_commands::format_code_command(
                args, None, None,
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod i18n_tests {
    use super::*;

    #[test]
    fn slash_command_descriptions_localize_in_zh_cn() {
        code_i18n::set_language(code_i18n::Language::ZhCn);
        assert_eq!(SlashCommand::Settings.description(), "集中管理所有设置");
        assert_eq!(SlashCommand::Chrome.description(), "连接到你的 Chrome 浏览器");
        assert_eq!(SlashCommand::Lang.description(), "切换界面语言（en/zh-CN）");
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter()
        .filter(|c| c.is_available())
        .map(|c| (c.command(), c))
        .collect()
}

/// Process a message that might contain a slash command.
/// Returns either the expanded prompt (for prompt-expanding commands) or the original message.
pub fn process_slash_command_message(message: &str) -> ProcessedCommand {
    let trimmed = message.trim();

    if trimmed.is_empty() {
        return ProcessedCommand::NotCommand(message.to_string());
    }

    let has_slash = trimmed.starts_with('/');
    let command_portion = if has_slash { &trimmed[1..] } else { trimmed };
    let mut parts = command_portion.splitn(2, |c: char| c.is_whitespace());
    let command_str = parts.next().unwrap_or("");
    let args_raw = parts.next().map(|s| s.trim()).unwrap_or("");
    let canonical_command = command_str.to_ascii_lowercase();

    if matches!(canonical_command.as_str(), "quit" | "exit") {
        if !has_slash && !args_raw.is_empty() {
            return ProcessedCommand::NotCommand(message.to_string());
        }

        let command_text = if args_raw.is_empty() {
            format!("/{}", SlashCommand::Quit.command())
        } else {
            format!("/{} {}", SlashCommand::Quit.command(), args_raw)
        };

        return ProcessedCommand::RegularCommand(SlashCommand::Quit, command_text);
    }

    if !has_slash {
        return ProcessedCommand::NotCommand(message.to_string());
    }

    // Try to parse the command
    if let Ok(command) = canonical_command.parse::<SlashCommand>() {
        if !command.is_available() {
            let command_name = command.command();
            let message = match command {
                SlashCommand::Demo => {
                    format!("Error: /{command_name} is only available in dev or perf builds.")
                }
                _ => format!("Error: /{command_name} is not available in this build."),
            };
            return ProcessedCommand::Error(message);
        }

        // Check if it's a prompt-expanding command
        if command.is_prompt_expanding() {
            if args_raw.is_empty() && command.requires_arguments() {
                return ProcessedCommand::Error(format!(
                    "Error: /{} requires a task description. Usage: /{} <task>",
                    command.command(),
                    command.command()
                ));
            }

            if let Some(expanded) = command.expand_prompt(args_raw) {
                return ProcessedCommand::ExpandedPrompt(expanded);
            }
        }

        let command_text = if args_raw.is_empty() {
            format!("/{}", command.command())
        } else {
            format!("/{} {}", command.command(), args_raw)
        };

        // It's a regular command, return it as-is with the canonical text
        ProcessedCommand::RegularCommand(command, command_text)
    } else {
        // Unknown command
        ProcessedCommand::NotCommand(message.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum ProcessedCommand {
    /// The message was expanded from a prompt-expanding slash command
    ExpandedPrompt(String),
    /// A regular slash command that should be handled by the TUI. The `String`
    /// contains the canonical command text (with leading slash and trimmed args).
    RegularCommand(SlashCommand, String),
    /// Not a slash command, just a regular message
    #[allow(dead_code)]
    NotCommand(String),
    /// Error processing the command
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_command_with_newline_arguments_is_recognized() {
        let msg = "/plan\nBuild a release plan\n- tighten scope";
        match process_slash_command_message(msg) {
            ProcessedCommand::ExpandedPrompt(prompt) => {
                assert!(prompt.contains("Build a release plan"));
                assert!(prompt.contains("tighten scope"));
            }
            other => panic!("expected ExpandedPrompt, got {:?}", other),
        }
    }

    #[test]
    fn auto_command_with_newline_arguments_is_regular_command() {
        let msg = "/auto\ninspect the failing build";
        match process_slash_command_message(msg) {
            ProcessedCommand::RegularCommand(SlashCommand::Auto, command_text) => {
                assert!(command_text.contains("inspect the failing build"));
            }
            other => panic!("expected RegularCommand, got {:?}", other),
        }
    }
}
