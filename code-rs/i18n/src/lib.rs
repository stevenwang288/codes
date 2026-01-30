use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::SystemTime;

use once_cell::sync::Lazy;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    ZhCn,
}

impl Language {
    pub fn as_bcp47(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::ZhCn => "zh-CN",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_bcp47())
    }
}

// Fork policy: default to Simplified Chinese unless explicitly overridden.
static CURRENT_LANGUAGE: AtomicU8 = AtomicU8::new(Language::ZhCn as u8);
static LANGUAGE_INIT: OnceLock<()> = OnceLock::new();

pub fn current_language() -> Language {
    init_language_from_env();
    match CURRENT_LANGUAGE.load(Ordering::Relaxed) {
        x if x == Language::ZhCn as u8 => Language::ZhCn,
        _ => Language::En,
    }
}

pub fn set_language(language: Language) {
    if language != Language::ZhCn {
        return;
    }
    CURRENT_LANGUAGE.store(language as u8, Ordering::Relaxed);
    persist_language(language);
    let _ = LANGUAGE_INIT.set(());
}

#[cfg(feature = "test-helpers")]
static TEST_LANGUAGE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[cfg(feature = "test-helpers")]
pub fn set_language_for_tests(language: Language) {
    CURRENT_LANGUAGE.store(language as u8, Ordering::Relaxed);
    let _ = LANGUAGE_INIT.set(());
}

#[cfg(feature = "test-helpers")]
pub fn with_test_language<R>(language: Language, f: impl FnOnce() -> R) -> R {
    let _guard = TEST_LANGUAGE_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let previous = current_language();
    set_language_for_tests(language);
    let result = f();
    set_language_for_tests(previous);
    result
}

fn init_language_from_env() {
    LANGUAGE_INIT.get_or_init(|| {
        let env = std::env::var("CODEX_LANG")
            .ok()
            .or_else(|| std::env::var("OPENCODE_LANGUAGE").ok());
        if let Some(raw) = env {
            if let Some(lang) = parse_language(&raw) {
                if lang == Language::ZhCn {
                    CURRENT_LANGUAGE.store(lang as u8, Ordering::Relaxed);
                }
                return;
            }
        }

        if let Some(lang) = read_language_from_file() {
            if lang == Language::ZhCn {
                CURRENT_LANGUAGE.store(lang as u8, Ordering::Relaxed);
            }
        }
    });
}

pub fn parse_language(raw: &str) -> Option<Language> {
    let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
    match normalized.as_str() {
        "en" | "en-us" | "en-gb" => Some(Language::En),
        "zh" | "zh-cn" | "zh-hans" => Some(Language::ZhCn),
        _ => None,
    }
}

pub fn language_name(ui_language: Language, target: Language) -> &'static str {
    match target {
        Language::En => tr(ui_language, "language.name.en"),
        Language::ZhCn => tr(ui_language, "language.name.zh_cn"),
    }
}

pub fn language_self_name(target: Language) -> &'static str {
    match target {
        Language::En => "English",
        Language::ZhCn => "简体中文",
    }
}

pub fn tr_plain(key: &'static str) -> &'static str {
    tr(current_language(), key)
}

pub fn tr(language: Language, key: &'static str) -> &'static str {
    if let Some(value) = lookup_string(language, key) {
        return value;
    }
    key
}

pub fn tr_args(language: Language, key: &'static str, args: &[(&str, &str)]) -> String {
    let template = tr(language, key);
    interpolate(template, args)
}

fn lookup_string(language: Language, key: &'static str) -> Option<&'static str> {
    let requested = catalog(language).strings.get(key).map(String::as_str);
    if requested.is_some() {
        return requested;
    }
    let fallback = catalog(Language::En).strings.get(key).map(String::as_str);
    if fallback.is_some() {
        log_missing_if_needed(language, key, fallback.unwrap_or_default());
    }
    fallback
}

fn catalog(language: Language) -> &'static Catalog {
    match language {
        Language::En => EN_CATALOG.get_or_init(|| Catalog::from_json(include_str!("../assets/en.json"))),
        Language::ZhCn => ZH_CATALOG.get_or_init(|| Catalog::from_json(include_str!("../assets/zh-CN.json"))),
    }
}

static EN_CATALOG: OnceLock<Catalog> = OnceLock::new();
static ZH_CATALOG: OnceLock<Catalog> = OnceLock::new();

#[derive(Debug, Default)]
struct Catalog {
    strings: HashMap<String, String>,
}

impl Catalog {
    fn from_json(raw: &str) -> Self {
        let parsed: HashMap<String, serde_json::Value> =
            serde_json::from_str(raw).unwrap_or_else(|err| panic!("i18n JSON must parse: {err}"));
        let mut catalog = Catalog::default();
        for (key, value) in parsed {
            let serde_json::Value::String(text) = value else {
                panic!("i18n JSON values must be strings; keys must be flat");
            };
            let previous = catalog.strings.insert(key, text);
            if previous.is_some() {
                panic!("duplicate i18n key");
            }
        }
        catalog
    }
}

fn interpolate(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("${") {
        let (before, after_start) = rest.split_at(start);
        out.push_str(before);
        let after_start = &after_start[2..];
        if let Some(end) = after_start.find('}') {
            let key = &after_start[..end];
            if let Some((_, value)) = args.iter().find(|(name, _)| *name == key) {
                out.push_str(value);
            } else {
                out.push_str("${");
                out.push_str(key);
                out.push('}');
            }
            rest = &after_start[end + 1..];
        } else {
            out.push_str("${");
            out.push_str(after_start);
            return out;
        }
    }
    out.push_str(rest);
    out
}

static LOGGED_MISSING_KEYS: Lazy<Mutex<HashSet<String>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

fn log_missing_if_needed(requested_language: Language, key: &'static str, fallback_text: &str) {
    if requested_language == Language::En {
        return;
    }

    let Ok(mut logged) = LOGGED_MISSING_KEYS.lock() else {
        return;
    };
    if !logged.insert(key.to_string()) {
        return;
    }

    let Some(path) = missing_log_path() else {
        return;
    };

    let ts_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let line = serde_json::json!({
        "ts_ms": ts_ms,
        "app": "code",
        "missing_in": requested_language.as_bcp47(),
        "key": key,
        "fallback_text": fallback_text,
    })
    .to_string();

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{line}");
    }
}

fn missing_log_path() -> Option<PathBuf> {
    let base = std::env::var_os("CODE_HOME")
        .or_else(|| std::env::var_os("CODEX_HOME"))
        .map(PathBuf::from)?;
    Some(base.join("i18n-missing.jsonl"))
}

fn read_language_from_file() -> Option<Language> {
    let path = persisted_language_path()?;
    let raw = std::fs::read_to_string(path).ok()?;
    parse_language(raw.trim())
}

fn persist_language(language: Language) {
    let Some(path) = persisted_language_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, language.as_bcp47());
}

fn persisted_language_path() -> Option<PathBuf> {
    let base = std::env::var_os("CODE_HOME")
        .or_else(|| std::env::var_os("CODEX_HOME"))
        .map(PathBuf::from)?;
    Some(base.join("ui-language.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_replaces_placeholders() {
        let rendered = interpolate("Hello ${name}", &[("name", "World")]);
        assert_eq!(rendered, "Hello World");
    }

    #[test]
    fn locale_files_have_same_keys() {
        let en = catalog(Language::En)
            .strings
            .keys()
            .cloned()
            .collect::<HashSet<_>>();
        let zh = catalog(Language::ZhCn)
            .strings
            .keys()
            .cloned()
            .collect::<HashSet<_>>();
        assert_eq!(en, zh);
    }
}
