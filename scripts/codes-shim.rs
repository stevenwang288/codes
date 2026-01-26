use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, SystemTime};

const GIT_BASH_EXE: &str = r"C:\Program Files\Git\usr\bin\bash.exe";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("[codes] ERROR: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let repo_root = resolve_repo_root()?;

    let use_global_home = match std::env::var("CODES_USE_GLOBAL_HOME") {
        Ok(v) if matches!(v.trim(), "0" | "false" | "no") => false,
        Ok(_) => true,
        Err(_) => true,
    };

    let code_home = if use_global_home {
        global_code_home_dir()?
    } else {
        repo_root.join(".codes-home")
    };

    let repo_last_built_file = repo_last_built_file(&repo_root);

    ensure_dir(&code_home)?;
    write_last_repo_root(&repo_root)?;

    // Prevent runaway loops from spawning dozens of windows/processes.
    // This can happen when a launcher is invoked repeatedly (e.g., shell retry loops,
    // stale shortcuts, or external wrappers). We keep this conservative and allow
    // overriding via CODES_NO_LOCK=1.
    let _lock_guard = if std::env::var_os("CODES_NO_LOCK").is_some() {
        None
    } else {
        Some(acquire_startup_lock()?)
    };

    std::env::set_var("CODE_HOME", &code_home);
    // Align with upstream conventions: some scripts/tools write cache markers under CODEX_HOME.
    std::env::set_var("CODEX_HOME", &code_home);
    std::env::set_var("CODE_AUTO_TRUST", "1");

    if std::env::var_os("CODEX_LANG").is_none() {
        let persisted = fs::read_to_string(code_home.join("ui-language.txt"))
            .ok()
            .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
            .filter(|s| !s.is_empty());
        std::env::set_var("CODEX_LANG", persisted.as_deref().unwrap_or("zh-CN"));
    }

    if let Some(first) = args.first().and_then(|a| a.to_str()) {
        if first.eq_ignore_ascii_case("which") {
            return Ok(run_which(&repo_root, &code_home, &repo_last_built_file, use_global_home));
        }
    }

    let mut force_rebuild = false;
    let mut collect_missing = false;
    let mut passthrough_args: Vec<OsString> = Vec::new();
    for arg in args {
        match arg.to_str() {
            Some("-z") => collect_missing = true,
            Some("--rebuild") | Some("build") => force_rebuild = true,
            _ => passthrough_args.push(arg),
        }
    }
    if collect_missing {
        std::env::set_var("CODE_I18N_COLLECT_MISSING", "1");
    }

    if !force_rebuild {
        let last_built_file = if use_global_home {
            &repo_last_built_file
        } else {
            &code_home.join("last-built-bin.txt")
        };
        if let Some(code_exe) = cached_code_exe(&repo_root, last_built_file)? {
            return exec(code_exe, passthrough_args);
        }
    }

    // Rebuild path.
    if !Path::new(GIT_BASH_EXE).exists() {
        return Err(format!("Git Bash not found at \"{GIT_BASH_EXE}\""));
    }

    let repo_msys = to_msys_path(&repo_root)?;
    let bash_script = if passthrough_args.is_empty() {
        format!("cd \"{repo_msys}\"; ./build-fast.sh run")
    } else {
        format!("cd \"{repo_msys}\"; ./build-fast.sh")
    };

    let status = Command::new(GIT_BASH_EXE)
        .arg("-lc")
        .arg(bash_script)
        .status()
        .map_err(|e| format!("failed to run build-fast.sh: {e}"))?;
    if !status.success() {
        return Ok(ExitCode::from(status.code().unwrap_or(1) as u8));
    }

    // Cache the last-built binary per repo when using a shared global CODE_HOME.
    if use_global_home {
        let global_last_built = code_home.join("last-built-bin.txt");
        if let Ok(text) = fs::read_to_string(&global_last_built) {
            let raw = text.lines().next().unwrap_or("").trim();
            if !raw.is_empty() {
                let _ = fs::write(&repo_last_built_file, format!("{raw}\n"));
            }
        }
    }

    if passthrough_args.is_empty() {
        // build-fast.sh run already launched the TUI.
        return Ok(ExitCode::from(0));
    }

    let last_built_file = if use_global_home {
        &repo_last_built_file
    } else {
        &code_home.join("last-built-bin.txt")
    };
    if let Some(code_exe) = cached_code_exe(&repo_root, last_built_file)? {
        exec(code_exe, passthrough_args)
    } else {
        Err("build succeeded but no cached code.exe was found".to_string())
    }
}

struct StartupLock {
    path: PathBuf,
}

impl Drop for StartupLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn parse_lock_pid(contents: &str) -> Option<u32> {
    contents
        .lines()
        .find_map(|line| line.trim().strip_prefix("pid="))
        .and_then(|pid| pid.trim().parse::<u32>().ok())
}

#[cfg(windows)]
fn pid_is_running(pid: u32) -> bool {
    let pid_filter = format!("PID eq {pid}");
    let Ok(output) = Command::new("tasklist")
        .args(["/FI", pid_filter.as_str(), "/NH"])
        .output()
    else {
        // If we cannot check, err on the side of safety and treat it as running.
        return true;
    };
    if !output.status.success() {
        return true;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(&pid.to_string())
}

#[cfg(not(windows))]
fn pid_is_running(_pid: u32) -> bool {
    true
}

fn acquire_startup_lock() -> Result<StartupLock, String> {
    let dir = state_dir();
    ensure_dir(&dir)?;
    let path = dir.join("codes.lock");

    // Best-effort stale lock cleanup:
    // - If the PID recorded in the lock isn't running, clear immediately.
    // - Otherwise, only clear if the file is old enough (protects against rapid relaunch loops).
    let maybe_clear_stale = |path: &Path| {
        if let Ok(text) = fs::read_to_string(path) {
            if let Some(pid) = parse_lock_pid(&text) {
                if !pid_is_running(pid) {
                    let _ = fs::remove_file(path);
                    return;
                }
            }
        }
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                if SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or(Duration::from_secs(0))
                    > Duration::from_secs(10 * 60)
                {
                    let _ = fs::remove_file(path);
                }
            }
        }
    };

    for attempt in 0..2 {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                use std::io::Write;
                let pid = std::process::id();
                let _ = writeln!(file, "pid={pid}");
                return Ok(StartupLock { path });
            }
            Err(_) if attempt == 0 => {
                maybe_clear_stale(&path);
                continue;
            }
            Err(_) => {
                return Err("检测到另一个 codes 正在启动/运行（为避免连开窗口已阻止重复启动）。请先关闭旧的 codes/Code 窗口再重试。".to_string());
            }
        }
    }

    Err("failed to acquire startup lock".to_string())
}

fn run_which(repo_root: &Path, code_home: &Path, repo_last_built: &Path, use_global_home: bool) -> ExitCode {
    println!("[codes] repo-root: \"{}\"", repo_root.display());
    println!("[codes] CODE_HOME: \"{}\"", code_home.display());
    println!("[codes] mode: {}", if use_global_home { "global" } else { "local" });
    println!(
        "[codes] CODEX_LANG: \"{}\"",
        std::env::var("CODEX_LANG").unwrap_or_else(|_| "".to_string())
    );
    println!("[codes] repo-last-built-file: \"{}\"", repo_last_built.display());

    let ui_language = code_home.join("ui-language.txt");
    if let Ok(text) = fs::read_to_string(&ui_language) {
        let value = text.lines().next().unwrap_or("").trim();
        println!("[codes] ui-language.txt: {value}");
    } else {
        println!("[codes] ui-language.txt: (missing)");
    }

    let last_built = code_home.join("last-built-bin.txt");
    if let Ok(text) = fs::read_to_string(&last_built) {
        let value = text.lines().next().unwrap_or("").trim();
        println!("[codes] last-built-bin.txt: {value}");
    } else {
        println!("[codes] last-built-bin.txt: (missing)");
    }

    if let Ok(text) = fs::read_to_string(repo_last_built) {
        let value = text.lines().next().unwrap_or("").trim();
        println!("[codes] repo-last-built: {value}");
    } else {
        println!("[codes] repo-last-built: (missing)");
    }

    ExitCode::from(0)
}

fn cached_code_exe(repo_root: &Path, last_built_file: &Path) -> Result<Option<PathBuf>, String> {
    // Fast path: local target dir (rare on Windows in this repo).
    let fast_bin = repo_root.join("code-rs").join("target").join("dev-fast").join("code.exe");
    if fast_bin.exists() {
        return Ok(Some(fast_bin));
    }

    let Ok(text) = fs::read_to_string(last_built_file) else {
        return Ok(None);
    };
    let raw = text.lines().next().unwrap_or("").trim();
    if raw.is_empty() {
        return Ok(None);
    }

    let Some(mut candidate) = msys_path_to_windows(raw) else {
        return Ok(None);
    };
    if !candidate.exists() {
        let with_exe = candidate.with_extension("exe");
        if with_exe.exists() {
            candidate = with_exe;
        }
    }
    if !candidate.exists() {
        return Ok(None);
    }

    Ok(Some(candidate))
}

fn exec(program: PathBuf, args: Vec<OsString>) -> Result<ExitCode, String> {
    let status = Command::new(&program)
        .args(args)
        .status()
        .map_err(|e| format!("failed to run \"{}\": {e}", program.display()))?;
    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}

fn resolve_repo_root() -> Result<PathBuf, String> {
    if let Some(root) = env_path("CODES_REPO_ROOT")
        .or_else(|| env_path("CODE_REPO_ROOT"))
        .filter(|p| has_build_fast(p))
    {
        return Ok(root);
    }

    if let Ok(cwd) = std::env::current_dir() {
        if let Some(root) = git_toplevel(&cwd).filter(|p| has_build_fast(p)) {
            return Ok(root);
        }
        if let Some(root) = search_up_for_build_fast(&cwd) {
            return Ok(root);
        }
    }

    if let Some(root) = read_last_repo_root().filter(|p| has_build_fast(p)) {
        return Ok(root);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if has_build_fast(dir) {
                return Ok(dir.to_path_buf());
            }
        }
    }

    Err("Repo root not found (missing build-fast.sh). Set CODES_REPO_ROOT.".to_string())
}

fn global_code_home_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| "USERPROFILE is not set".to_string())?;
    Ok(home.join(".code"))
}

fn repo_last_built_file(repo_root: &Path) -> PathBuf {
    let hash = fnv1a_64(&repo_root.to_string_lossy());
    state_dir().join(format!("last-built-{hash:016x}.txt"))
}

fn fnv1a_64(text: &str) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn has_build_fast(dir: &Path) -> bool {
    dir.join("build-fast.sh").exists()
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from)
}

fn git_toplevel(cwd: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        None
    } else {
        Some(PathBuf::from(line))
    }
}

fn search_up_for_build_fast(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if has_build_fast(dir) {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn state_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(dir).join("codes");
    }
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        return PathBuf::from(profile).join("AppData").join("Local").join("codes");
    }
    PathBuf::from(".").join("codes")
}

fn last_repo_file() -> PathBuf {
    state_dir().join("repo-root.txt")
}

fn read_last_repo_root() -> Option<PathBuf> {
    fs::read_to_string(last_repo_file())
        .ok()
        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

fn write_last_repo_root(repo_root: &Path) -> Result<(), String> {
    let dir = state_dir();
    ensure_dir(&dir)?;
    fs::write(last_repo_file(), format!("{}\n", repo_root.display()))
        .map_err(|e| format!("failed to write repo-root.txt: {e}"))
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("failed to create \"{}\": {e}", path.display()))
}

fn msys_path_to_windows(raw: &str) -> Option<PathBuf> {
    let raw = raw.trim();
    let Some(rest) = raw.strip_prefix('/') else {
        return None;
    };
    let mut parts = rest.splitn(2, '/');
    let drive = parts.next()?.chars().next()?.to_ascii_uppercase();
    let tail = parts.next().unwrap_or("");
    let mut out = String::new();
    out.push(drive);
    out.push(':');
    out.push('\\');
    out.push_str(&tail.replace('/', "\\"));
    Some(PathBuf::from(out))
}

fn to_msys_path(path: &Path) -> Result<String, String> {
    let s = path.to_string_lossy();
    if s.len() >= 2 && s.as_bytes()[1] == b':' {
        let drive = s.chars().next().unwrap_or('c').to_ascii_lowercase();
        let rest = &s[2..];
        let rest = rest.trim_start_matches(['\\', '/']);
        let rest = rest.replace('\\', "/");
        Ok(format!("/{drive}/{rest}"))
    } else {
        Err(format!("unsupported repo root path: \"{}\"", path.display()))
    }
}
