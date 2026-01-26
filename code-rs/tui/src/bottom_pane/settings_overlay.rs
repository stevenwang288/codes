use code_i18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsSection {
    Model,
    Theme,
    Updates,
    Accounts,
    Agents,
    Prompts,
    Skills,
    AutoDrive,
    Review,
    Planning,
    Validation,
    Limits,
    Chrome,
    Mcp,
    Notifications,
}

impl SettingsSection {
    pub(crate) const ALL: [SettingsSection; 15] = [
        SettingsSection::Model,
        SettingsSection::Theme,
        SettingsSection::Updates,
        SettingsSection::Accounts,
        SettingsSection::Agents,
        SettingsSection::Prompts,
        SettingsSection::Skills,
        SettingsSection::AutoDrive,
        SettingsSection::Review,
        SettingsSection::Planning,
        SettingsSection::Validation,
        SettingsSection::Chrome,
        SettingsSection::Mcp,
        SettingsSection::Notifications,
        SettingsSection::Limits,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            SettingsSection::Model => code_i18n::tr_plain("tui.settings.section.model"),
            SettingsSection::Theme => code_i18n::tr_plain("tui.settings.section.theme"),
            SettingsSection::Planning => code_i18n::tr_plain("tui.settings.section.planning"),
            SettingsSection::Updates => code_i18n::tr_plain("tui.settings.section.updates"),
            SettingsSection::Accounts => code_i18n::tr_plain("tui.settings.section.accounts"),
            SettingsSection::Agents => code_i18n::tr_plain("tui.settings.section.agents"),
            SettingsSection::AutoDrive => code_i18n::tr_plain("tui.settings.section.auto_drive"),
            SettingsSection::Review => code_i18n::tr_plain("tui.settings.section.review"),
            SettingsSection::Validation => code_i18n::tr_plain("tui.settings.section.validation"),
            SettingsSection::Limits => code_i18n::tr_plain("tui.settings.section.limits"),
            SettingsSection::Chrome => code_i18n::tr_plain("tui.settings.section.chrome"),
            SettingsSection::Mcp => code_i18n::tr_plain("tui.settings.section.mcp"),
            SettingsSection::Notifications => code_i18n::tr_plain("tui.settings.section.notifications"),
            SettingsSection::Prompts => code_i18n::tr_plain("tui.settings.section.prompts"),
            SettingsSection::Skills => code_i18n::tr_plain("tui.settings.section.skills"),
        }
    }

    pub(crate) fn help_line(self) -> &'static str {
        match self {
            SettingsSection::Model => code_i18n::tr_plain("tui.settings.help.model"),
            SettingsSection::Theme => code_i18n::tr_plain("tui.settings.help.theme"),
            SettingsSection::Planning => code_i18n::tr_plain("tui.settings.help.planning"),
            SettingsSection::Updates => code_i18n::tr_plain("tui.settings.help.updates"),
            SettingsSection::Accounts => code_i18n::tr_plain("tui.settings.help.accounts"),
            SettingsSection::Agents => code_i18n::tr_plain("tui.settings.help.agents"),
            SettingsSection::AutoDrive => code_i18n::tr_plain("tui.settings.help.auto_drive"),
            SettingsSection::Review => code_i18n::tr_plain("tui.settings.help.review"),
            SettingsSection::Validation => code_i18n::tr_plain("tui.settings.help.validation"),
            SettingsSection::Limits => code_i18n::tr_plain("tui.settings.help.limits"),
            SettingsSection::Chrome => code_i18n::tr_plain("tui.settings.help.chrome"),
            SettingsSection::Mcp => code_i18n::tr_plain("tui.settings.help.mcp"),
            SettingsSection::Notifications => code_i18n::tr_plain("tui.settings.help.notifications"),
            SettingsSection::Prompts => code_i18n::tr_plain("tui.settings.help.prompts"),
            SettingsSection::Skills => code_i18n::tr_plain("tui.settings.help.skills"),
        }
    }

    pub(crate) fn placeholder(self) -> &'static str {
        match self {
            SettingsSection::Model => code_i18n::tr_plain("tui.settings.placeholder.model"),
            SettingsSection::Theme => code_i18n::tr_plain("tui.settings.placeholder.theme"),
            SettingsSection::Planning => code_i18n::tr_plain("tui.settings.placeholder.planning"),
            SettingsSection::Updates => code_i18n::tr_plain("tui.settings.placeholder.updates"),
            SettingsSection::Accounts => code_i18n::tr_plain("tui.settings.placeholder.accounts"),
            SettingsSection::Agents => code_i18n::tr_plain("tui.settings.placeholder.agents"),
            SettingsSection::AutoDrive => code_i18n::tr_plain("tui.settings.placeholder.auto_drive"),
            SettingsSection::Review => code_i18n::tr_plain("tui.settings.placeholder.review"),
            SettingsSection::Validation => code_i18n::tr_plain("tui.settings.placeholder.validation"),
            SettingsSection::Limits => code_i18n::tr_plain("tui.settings.placeholder.limits"),
            SettingsSection::Chrome => code_i18n::tr_plain("tui.settings.placeholder.chrome"),
            SettingsSection::Mcp => code_i18n::tr_plain("tui.settings.placeholder.mcp"),
            SettingsSection::Notifications => code_i18n::tr_plain("tui.settings.placeholder.notifications"),
            SettingsSection::Prompts => code_i18n::tr_plain("tui.settings.placeholder.prompts"),
            SettingsSection::Skills => code_i18n::tr_plain("tui.settings.placeholder.skills"),
        }
    }

    pub(crate) fn from_hint(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "model" | "models" => Some(SettingsSection::Model),
            "skill" | "skills" => Some(SettingsSection::Skills),
            "theme" | "themes" => Some(SettingsSection::Theme),
            "planning" | "plan" => Some(SettingsSection::Planning),
            "update" | "updates" => Some(SettingsSection::Updates),
            "account" | "accounts" | "auth" => Some(SettingsSection::Accounts),
            "agent" | "agents" => Some(SettingsSection::Agents),
            "auto" | "autodrive" | "drive" => Some(SettingsSection::AutoDrive),
            "review" | "reviews" => Some(SettingsSection::Review),
            "validation" | "validate" => Some(SettingsSection::Validation),
            "limit" | "limits" | "usage" => Some(SettingsSection::Limits),
            "chrome" | "browser" => Some(SettingsSection::Chrome),
            "mcp" => Some(SettingsSection::Mcp),
            "notification" | "notifications" | "notify" | "notif" => Some(SettingsSection::Notifications),
            _ => None,
        }
    }
}
