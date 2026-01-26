use code_app_server_protocol::AuthMode;
use code_common::CliConfigOverrides;
use code_core::CodexAuth;
use code_core::auth::CLIENT_ID;
use code_core::auth::OPENAI_API_KEY_ENV_VAR;
use code_core::auth::login_with_api_key;
use code_core::auth::logout;
use code_core::config::Config;
use code_core::config::ConfigOverrides;
use code_login::ServerOptions;
use code_login::run_device_code_login;
use code_login::run_login_server;
use std::env;
use std::io::IsTerminal;
use std::io::Read;
use std::path::PathBuf;

pub async fn login_with_chatgpt(code_home: PathBuf, originator: String) -> std::io::Result<()> {
    let opts = ServerOptions::new(code_home, CLIENT_ID.to_string(), originator);
    let server = run_login_server(opts)?;

    let ui_language = code_i18n::current_language();
    eprintln!(
        "{}",
        code_i18n::tr_args(
            ui_language,
            "cli.login.local_server_started",
            &[
                ("port", &server.actual_port.to_string()),
                ("url", server.auth_url.as_str())
            ],
        )
    );

    server.block_until_done().await
}

pub async fn run_login_with_chatgpt(cli_config_overrides: CliConfigOverrides) -> ! {
    let config = load_config_or_exit(cli_config_overrides);

    match login_with_chatgpt(
        config.code_home,
        config.responses_originator_header.clone(),
    )
    .await
    {
        Ok(_) => {
            eprintln!("{}", code_i18n::tr_plain("cli.login.success"));
            std::process::exit(0);
        }
        Err(e) => {
            let ui_language = code_i18n::current_language();
            eprintln!(
                "{}",
                code_i18n::tr_args(ui_language, "cli.login.error", &[("error", &e.to_string())])
            );
            std::process::exit(1);
        }
    }
}

pub async fn run_login_with_api_key(
    cli_config_overrides: CliConfigOverrides,
    api_key: String,
) -> ! {
    let config = load_config_or_exit(cli_config_overrides);

    match login_with_api_key(&config.code_home, &api_key) {
        Ok(_) => {
            eprintln!("{}", code_i18n::tr_plain("cli.login.success"));
            std::process::exit(0);
        }
        Err(e) => {
            let ui_language = code_i18n::current_language();
            eprintln!(
                "{}",
                code_i18n::tr_args(ui_language, "cli.login.error", &[("error", &e.to_string())])
            );
            std::process::exit(1);
        }
    }
}

pub fn read_api_key_from_stdin() -> String {
    let mut stdin = std::io::stdin();

    if stdin.is_terminal() {
        eprintln!("{}", code_i18n::tr_plain("cli.login.with_api_key_requires_stdin"));
        std::process::exit(1);
    }

    eprintln!("{}", code_i18n::tr_plain("cli.login.reading_api_key_from_stdin"));

    let mut buffer = String::new();
    if let Err(err) = stdin.read_to_string(&mut buffer) {
        let ui_language = code_i18n::current_language();
        eprintln!(
            "{}",
            code_i18n::tr_args(
                ui_language,
                "cli.login.read_api_key_from_stdin_failed",
                &[("error", &err.to_string())]
            )
        );
        std::process::exit(1);
    }

    let api_key = buffer.trim().to_string();
    if api_key.is_empty() {
        eprintln!("{}", code_i18n::tr_plain("cli.login.no_api_key_provided"));
        std::process::exit(1);
    }

    api_key
}

/// Login using the OAuth device code flow.
pub async fn run_login_with_device_code(
    cli_config_overrides: CliConfigOverrides,
    issuer_base_url: Option<String>,
    client_id: Option<String>,
) -> ! {
    let config = load_config_or_exit(cli_config_overrides);
    let mut opts = ServerOptions::new(
        config.code_home,
        client_id.unwrap_or(CLIENT_ID.to_string()),
        config.responses_originator_header.clone(),
    );
    if let Some(iss) = issuer_base_url {
        opts.issuer = iss;
    }
    match run_device_code_login(opts).await {
        Ok(()) => {
            eprintln!("{}", code_i18n::tr_plain("cli.login.success"));
            std::process::exit(0);
        }
        Err(e) => {
            let ui_language = code_i18n::current_language();
            eprintln!(
                "{}",
                code_i18n::tr_args(
                    ui_language,
                    "cli.login.device_code_error",
                    &[("error", &e.to_string())]
                )
            );
            std::process::exit(1);
        }
    }
}

pub async fn run_login_status(cli_config_overrides: CliConfigOverrides) -> ! {
    let config = load_config_or_exit(cli_config_overrides);

    match CodexAuth::from_code_home(
        &config.code_home,
        AuthMode::ApiKey,
        &config.responses_originator_header,
    ) {
        Ok(Some(auth)) => match auth.mode {
            AuthMode::ApiKey => match auth.get_token().await {
                Ok(api_key) => {
                    let ui_language = code_i18n::current_language();
                    let safe = safe_format_key(&api_key);
                    eprintln!(
                        "{}",
                        code_i18n::tr_args(
                            ui_language,
                            "cli.login.status.api_key",
                            &[("key", &safe)]
                        )
                    );

                    if let Ok(env_api_key) = env::var(OPENAI_API_KEY_ENV_VAR) {
                        if env_api_key == api_key {
                            eprintln!("{}", code_i18n::tr_plain("cli.login.status.api_key_from_env"));
                        }
                    }
                    std::process::exit(0);
                }
                Err(e) => {
                    let ui_language = code_i18n::current_language();
                    eprintln!(
                        "{}",
                        code_i18n::tr_args(
                            ui_language,
                            "cli.login.status.api_key_read_error",
                            &[("error", &e.to_string())]
                        )
                    );
                    std::process::exit(1);
                }
            },
            AuthMode::ChatGPT => {
                eprintln!("{}", code_i18n::tr_plain("cli.login.status.chatgpt"));
                std::process::exit(0);
            }
        },
        Ok(None) => {
            eprintln!("{}", code_i18n::tr_plain("cli.login.status.not_logged_in"));
            std::process::exit(1);
        }
        Err(e) => {
            let ui_language = code_i18n::current_language();
            eprintln!(
                "{}",
                code_i18n::tr_args(
                    ui_language,
                    "cli.login.status.error",
                    &[("error", &e.to_string())]
                )
            );
            std::process::exit(1);
        }
    }
}

pub async fn run_logout(cli_config_overrides: CliConfigOverrides) -> ! {
    let config = load_config_or_exit(cli_config_overrides);

    match logout(&config.code_home) {
        Ok(true) => {
            eprintln!("{}", code_i18n::tr_plain("cli.logout.success"));
            std::process::exit(0);
        }
        Ok(false) => {
            eprintln!("{}", code_i18n::tr_plain("cli.login.status.not_logged_in"));
            std::process::exit(0);
        }
        Err(e) => {
            let ui_language = code_i18n::current_language();
            eprintln!(
                "{}",
                code_i18n::tr_args(ui_language, "cli.logout.error", &[("error", &e.to_string())])
            );
            std::process::exit(1);
        }
    }
}

fn load_config_or_exit(cli_config_overrides: CliConfigOverrides) -> Config {
    let cli_overrides = match cli_config_overrides.parse_overrides() {
        Ok(v) => v,
        Err(e) => {
            let ui_language = code_i18n::current_language();
            eprintln!(
                "{}",
                code_i18n::tr_args(
                    ui_language,
                    "cli.config.parse_overrides_error",
                    &[("error", &e.to_string())]
                )
            );
            std::process::exit(1);
        }
    };

    let config_overrides = ConfigOverrides::default();
    match Config::load_with_cli_overrides(cli_overrides, config_overrides) {
        Ok(config) => config,
        Err(e) => {
            let ui_language = code_i18n::current_language();
            eprintln!(
                "{}",
                code_i18n::tr_args(
                    ui_language,
                    "cli.config.load_error",
                    &[("error", &e.to_string())]
                )
            );
            std::process::exit(1);
        }
    }
}

fn safe_format_key(key: &str) -> String {
    if key.len() <= 13 {
        return "***".to_string();
    }
    let prefix = &key[..8];
    let suffix = &key[key.len() - 5..];
    format!("{prefix}***{suffix}")
}

#[cfg(test)]
mod tests {
    use super::safe_format_key;

    #[test]
    fn formats_long_key() {
        let key = "sk-proj-1234567890ABCDE";
        assert_eq!(safe_format_key(key), "sk-proj-***ABCDE");
    }

    #[test]
    fn short_key_returns_stars() {
        let key = "sk-proj-12345";
        assert_eq!(safe_format_key(key), "***");
    }
}
