use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use ignore::WalkBuilder;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use serde::Deserialize;
use serde::Serialize;

#[derive(Parser)]
#[command(name = "code-i18n-scan")]
#[command(about = "扫描 Rust 源码中的 i18n key 与候选未国际化字符串", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 生成扫描结果（JSON）与翻译清单（Markdown）
    Scan {
        /// 扫描根目录（通常为 code-rs 工作区根）
        #[arg(long)]
        root: PathBuf,
        /// 输出 JSON（包含 key 使用与候选未国际化字符串）
        #[arg(long)]
        out_json: Option<PathBuf>,
        /// 输出 Markdown（翻译清单）
        #[arg(long)]
        out_md: Option<PathBuf>,
    },
    /// 用当前扫描结果生成 baseline（允许列表）
    UpdateBaseline {
        /// 扫描根目录（通常为 code-rs 工作区根）
        #[arg(long)]
        root: PathBuf,
        /// baseline 输出路径（建议 i18n/scan_baseline.json）
        #[arg(long)]
        baseline: PathBuf,
    },
    /// 对比 baseline 做门禁检查（用于 CI/build-fast）
    Check {
        /// 扫描根目录（通常为 code-rs 工作区根）
        #[arg(long)]
        root: PathBuf,
        /// baseline 路径（建议 i18n/scan_baseline.json）
        #[arg(long)]
        baseline: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaselineV1 {
    version: u32,
    allowed_raw_strings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ScanOutputV1 {
    version: u32,
    i18n_keys_used: Vec<String>,
    i18n_keys_missing_in_assets: Vec<String>,
    raw_string_candidates: Vec<String>,
    raw_string_candidates_missing_in_baseline: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan {
            root,
            out_json,
            out_md,
        } => {
            let report = scan(&root)?;
            let output = build_scan_output(&root, &report, None)?;
            if let Some(path) = out_json {
                write_json(&path, &output)?;
            }
            if let Some(path) = out_md {
                write_translation_checklist_md(&path, &output)?;
            }
            Ok(())
        }
        Command::UpdateBaseline { root, baseline } => {
            let report = scan(&root)?;
            let mut allowed = BTreeSet::new();
            for s in report.raw_string_candidates {
                allowed.insert(s);
            }
            let baseline_data = BaselineV1 {
                version: 1,
                allowed_raw_strings: allowed.into_iter().collect(),
            };
            write_json(&baseline, &baseline_data)?;
            Ok(())
        }
        Command::Check { root, baseline } => {
            let baseline_data = read_baseline(&baseline)?;
            let report = scan(&root)?;
            let output = build_scan_output(&root, &report, Some(&baseline_data))?;
            if !output.i18n_keys_missing_in_assets.is_empty() {
                bail!(
                    "i18n key 缺失（需要补充到 i18n/assets/*.json）：{}",
                    output.i18n_keys_missing_in_assets.join(", ")
                );
            }
            if !output.raw_string_candidates_missing_in_baseline.is_empty() {
                bail!(
                    "发现未纳入 baseline 的候选未国际化字符串：{}",
                    output
                        .raw_string_candidates_missing_in_baseline
                        .join(", ")
                );
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct ScanReport {
    i18n_keys_used: BTreeSet<String>,
    raw_string_candidates: BTreeSet<String>,
}

fn scan(root: &Path) -> Result<ScanReport> {
    let mut i18n_keys_used = BTreeSet::new();
    let mut raw_string_candidates = BTreeSet::new();

    let mut walker = WalkBuilder::new(root);
    walker.hidden(false);
    walker.standard_filters(true);
    let walker = walker.build();

    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => return Err(anyhow!(err)),
        };
        if !entry.file_type().is_some_and(|ty| ty.is_file()) {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_under_dir(path, root, "target") {
            continue;
        }

        let raw = match fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(err) => return Err(anyhow!(err)).with_context(|| format!("读取失败: {}", path.display())),
        };

        let stream = TokenStream::from_str(&raw)
            .map_err(|err| anyhow!(err.to_string()))
            .with_context(|| format!("Rust tokenization 失败: {}", path.display()))?;

        let file_i18n_keys = collect_i18n_keys(&stream);
        i18n_keys_used.extend(file_i18n_keys.iter().cloned());

        if is_in_raw_scan_scope(root, path) {
            for s in collect_string_literals(&stream) {
                if file_i18n_keys.contains(&s) {
                    continue;
                }
                if is_raw_string_candidate(&s) {
                    raw_string_candidates.insert(s);
                }
            }
        }
    }

    Ok(ScanReport {
        i18n_keys_used,
        raw_string_candidates,
    })
}

fn is_under_dir(path: &Path, root: &Path, dirname: &str) -> bool {
    let Ok(stripped) = path.strip_prefix(root) else {
        return false;
    };
    let mut components = stripped.components();
    matches!(components.next().and_then(|c| c.as_os_str().to_str()), Some(name) if name == dirname)
}

fn is_in_raw_scan_scope(root: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return false;
    };
    let rel = rel.to_string_lossy().replace('\\', "/");
    for prefix in ["tui/src/", "cli/src/", "exec/src/", "cloud-tasks/src/"] {
        if rel.starts_with(prefix) {
            return true;
        }
    }
    false
}

fn collect_i18n_keys(stream: &TokenStream) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    collect_i18n_keys_in_stream(stream.clone(), &mut keys);
    keys
}

fn collect_i18n_keys_in_stream(stream: TokenStream, keys: &mut BTreeSet<String>) {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    let mut idx = 0;
    while idx < tokens.len() {
        match &tokens[idx] {
            TokenTree::Group(group) => {
                collect_i18n_keys_in_stream(group.stream(), keys);
                idx += 1;
            }
            TokenTree::Ident(ident) if ident == "code_i18n" => {
                if let Some((func_name, args_group, consumed)) = match_code_i18n_call(&tokens, idx) {
                    if let Some(key) = extract_key_from_args(func_name, &args_group) {
                        keys.insert(key);
                    }
                    idx += consumed;
                } else {
                    idx += 1;
                }
            }
            _ => idx += 1,
        }
    }
}

fn match_code_i18n_call(
    tokens: &[TokenTree],
    start_idx: usize,
) -> Option<(&'static str, proc_macro2::Group, usize)> {
    let t1 = tokens.get(start_idx + 1)?;
    let t2 = tokens.get(start_idx + 2)?;
    let t3 = tokens.get(start_idx + 3)?;
    let t4 = tokens.get(start_idx + 4)?;

    if !matches!(t1, TokenTree::Punct(p) if p.as_char() == ':') {
        return None;
    }
    if !matches!(t2, TokenTree::Punct(p) if p.as_char() == ':') {
        return None;
    }

    let TokenTree::Ident(func_ident) = t3 else {
        return None;
    };
    let func = func_ident.to_string();
    let func_name: &'static str = match func.as_str() {
        "tr_plain" => "tr_plain",
        "tr" => "tr",
        "tr_args" => "tr_args",
        _ => return None,
    };

    let TokenTree::Group(group) = t4 else {
        return None;
    };
    if group.delimiter() != proc_macro2::Delimiter::Parenthesis {
        return None;
    }

    Some((func_name, group.clone(), 5))
}

fn extract_key_from_args(func_name: &str, args_group: &proc_macro2::Group) -> Option<String> {
    let args = split_args(args_group.stream());
    let key_arg_idx = match func_name {
        "tr_plain" => 0,
        "tr" | "tr_args" => 1,
        _ => return None,
    };
    let key_stream = args.get(key_arg_idx)?;
    extract_string_literal_value(key_stream)
}

fn split_args(stream: TokenStream) -> Vec<TokenStream> {
    let mut args = Vec::new();
    let mut current: Vec<TokenTree> = Vec::new();

    for tt in stream {
        match &tt {
            TokenTree::Punct(p) if p.as_char() == ',' => {
                let ts = TokenStream::from_iter(current.drain(..));
                if !ts.is_empty() {
                    args.push(ts);
                }
            }
            _ => current.push(tt),
        }
    }

    let ts = TokenStream::from_iter(current);
    if !ts.is_empty() {
        args.push(ts);
    }

    args
}

fn extract_string_literal_value(tokens: &TokenStream) -> Option<String> {
    for tt in tokens.clone() {
        match tt {
            TokenTree::Literal(lit) => {
                let raw = lit.to_string();
                if let Ok(parsed) = syn::parse_str::<syn::Lit>(&raw) {
                    if let syn::Lit::Str(s) = parsed {
                        return Some(s.value());
                    }
                }
            }
            TokenTree::Group(group) => {
                if let Some(value) = extract_string_literal_value(&group.stream()) {
                    return Some(value);
                }
            }
            _ => {}
        }
    }
    None
}

fn collect_string_literals(stream: &TokenStream) -> Vec<String> {
    let mut out = Vec::new();
    collect_string_literals_in_stream(stream.clone(), &mut out);
    out
}

fn collect_string_literals_in_stream(stream: TokenStream, out: &mut Vec<String>) {
    fn is_doc_attribute_group(group: &proc_macro2::Group) -> bool {
        if group.delimiter() != proc_macro2::Delimiter::Bracket {
            return false;
        }
        matches!(
            group.stream().into_iter().next(),
            Some(TokenTree::Ident(ident)) if ident == "doc"
        )
    }

    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    let mut idx = 0;
    while idx < tokens.len() {
        match &tokens[idx] {
            TokenTree::Literal(lit) => {
                let raw = lit.to_string();
                if let Ok(parsed) = syn::parse_str::<syn::Lit>(&raw) {
                    if let syn::Lit::Str(s) = parsed {
                        out.push(s.value());
                    }
                }
                idx += 1;
            }
            TokenTree::Group(group) => {
                collect_string_literals_in_stream(group.stream(), out);
                idx += 1;
            }
            TokenTree::Punct(p) if p.as_char() == '#' => {
                // Skip doc comments/attributes: Rust lowers `/// ...` into `#[doc = "..."]`.
                // Doc strings are not user-visible UI, so ignore them for raw-string gating.
                let mut j = idx + 1;
                if matches!(tokens.get(j), Some(TokenTree::Punct(p2)) if p2.as_char() == '!') {
                    j += 1;
                }
                if let Some(TokenTree::Group(group)) = tokens.get(j) {
                    if is_doc_attribute_group(group) {
                        idx = j + 1;
                        continue;
                    }
                }
                idx += 1;
            }
            _ => {
                idx += 1;
            }
        }
    }
}

fn is_raw_string_candidate(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.len() < 2 {
        return false;
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return false;
    }
    if trimmed.starts_with("--") {
        return false;
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let has_letter = trimmed.chars().any(|c| c.is_alphabetic());
    let has_cjk = trimmed.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
    if !has_letter && !has_cjk {
        return false;
    }

    let looks_like_lower_ident = trimmed
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if looks_like_lower_ident {
        return false;
    }

    let looks_like_env = trimmed
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if looks_like_env {
        return false;
    }

    true
}

fn read_baseline(path: &Path) -> Result<BaselineV1> {
    let raw = fs::read_to_string(path).with_context(|| format!("读取 baseline 失败: {}", path.display()))?;
    let baseline: BaselineV1 =
        serde_json::from_str(&raw).with_context(|| format!("解析 baseline JSON 失败: {}", path.display()))?;
    if baseline.version != 1 {
        bail!("不支持的 baseline 版本: {}", baseline.version);
    }
    Ok(baseline)
}

fn build_scan_output(root: &Path, report: &ScanReport, baseline: Option<&BaselineV1>) -> Result<ScanOutputV1> {
    let (en, zh) = read_assets(root)?;

    let mut missing_in_assets = Vec::new();
    for key in &report.i18n_keys_used {
        if !en.contains_key(key) || !zh.contains_key(key) {
            missing_in_assets.push(key.clone());
        }
    }
    missing_in_assets.sort();
    missing_in_assets.dedup();

    let mut raw_candidates: Vec<String> = report.raw_string_candidates.iter().cloned().collect();
    raw_candidates.sort();
    raw_candidates.dedup();

    let mut raw_missing_in_baseline = Vec::new();
    if let Some(baseline) = baseline {
        let allowed: BTreeSet<&str> = baseline.allowed_raw_strings.iter().map(String::as_str).collect();
        for s in &raw_candidates {
            if !allowed.contains(s.as_str()) {
                raw_missing_in_baseline.push(s.clone());
            }
        }
    }

    Ok(ScanOutputV1 {
        version: 1,
        i18n_keys_used: report.i18n_keys_used.iter().cloned().collect(),
        i18n_keys_missing_in_assets: missing_in_assets,
        raw_string_candidates: raw_candidates,
        raw_string_candidates_missing_in_baseline: raw_missing_in_baseline,
    })
}

fn read_assets(root: &Path) -> Result<(BTreeMap<String, String>, BTreeMap<String, String>)> {
    let en_path = root.join("i18n").join("assets").join("en.json");
    let zh_path = root.join("i18n").join("assets").join("zh-CN.json");
    let en = read_flat_json_map(&en_path)?;
    let zh = read_flat_json_map(&zh_path)?;
    Ok((en, zh))
}

fn read_flat_json_map(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = fs::read_to_string(path).with_context(|| format!("读取 i18n 文件失败: {}", path.display()))?;
    let parsed: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&raw).with_context(|| format!("解析 i18n JSON 失败: {}", path.display()))?;
    let mut map = BTreeMap::new();
    for (k, v) in parsed {
        let serde_json::Value::String(s) = v else {
            return Err(anyhow!("i18n JSON 必须是 flat 的 string map: {}", path.display()));
        };
        map.insert(k, s);
    }
    Ok(map)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("创建目录失败: {}", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(value).context("序列化 JSON 失败")?;
    fs::write(path, raw).with_context(|| format!("写入失败: {}", path.display()))?;
    Ok(())
}

fn write_translation_checklist_md(path: &Path, output: &ScanOutputV1) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("创建目录失败: {}", parent.display()))?;
    }

    let mut md = String::new();
    md.push_str("# 国际化翻译清单（自动生成）\n\n");
    md.push_str("## 概览\n\n");
    md.push_str(&format!("- i18n key 使用量：{}\n", output.i18n_keys_used.len()));
    md.push_str(&format!(
        "- i18n key 缺失量（需补充到 i18n/assets/*.json）：{}\n",
        output.i18n_keys_missing_in_assets.len()
    ));
    md.push_str(&format!(
        "- 候选未国际化字符串（扫描范围：tui/cli/exec/cloud-tasks）：{}\n",
        output.raw_string_candidates.len()
    ));
    md.push_str("\n");

    md.push_str("## 缺失的 i18n key\n\n");
    if output.i18n_keys_missing_in_assets.is_empty() {
        md.push_str("- （无）\n\n");
    } else {
        for key in &output.i18n_keys_missing_in_assets {
            md.push_str(&format!("- `{key}`\n"));
        }
        md.push('\n');
    }

    md.push_str("## 候选未国际化字符串（需迁移到 code-i18n）\n\n");
    if output.raw_string_candidates.is_empty() {
        md.push_str("- （无）\n");
    } else {
        for s in &output.raw_string_candidates {
            md.push_str(&format!("- `{}`\n", escape_md_inline_code(s)));
        }
    }
    md.push('\n');

    fs::write(path, md).with_context(|| format!("写入失败: {}", path.display()))?;
    Ok(())
}

fn escape_md_inline_code(s: &str) -> String {
    s.replace('`', "\\`")
}
