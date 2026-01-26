use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use proc_macro2::TokenTree;
use serde::Serialize;
use syn::visit::Visit;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
#[command(name = "code-i18n-scan")]
#[command(about = "Scan Rust sources for user-visible string literals (AST/token based).")]
struct Args {
    /// Root directory to scan.
    #[arg(long, default_value = ".")]
    root: PathBuf,

    /// Subpaths under root to include (repeatable). If omitted, scans all under root.
    #[arg(long)]
    include: Vec<PathBuf>,

    /// Maximum number of findings to emit (0 = unlimited).
    #[arg(long, default_value_t = 0)]
    limit: usize,
}

#[derive(Debug, Serialize)]
struct Finding {
    file: String,
    literal: String,
    action: &'static str,
    reason: &'static str,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let root = args.root;
    let includes = if args.include.is_empty() {
        vec![root.clone()]
    } else {
        args.include
            .into_iter()
            .map(|p| root.join(p))
            .collect::<Vec<_>>()
    };

    let mut emitted = 0usize;
    for include in includes {
        if !include.exists() {
            continue;
        }
        for entry in WalkDir::new(&include).into_iter().filter_map(Result::ok) {
            if args.limit != 0 && emitted >= args.limit {
                return Ok(());
            }
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let raw = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;

            // syn does not expand macros; still useful for structured literals.
            // For macros like println!/bail!, we additionally scan token trees.
            let file = match syn::parse_file(&raw) {
                Ok(f) => f,
                Err(_) => {
                    // Fall back to token-only scan when syn parsing fails.
                    scan_tokens(path, &raw, &mut emitted, args.limit)?;
                    continue;
                }
            };

            let mut visitor = LiteralVisitor::new(path);
            visitor.visit_file(&file);

            for literal in visitor.literals {
                if args.limit != 0 && emitted >= args.limit {
                    return Ok(());
                }
                let (action, reason) = classify_literal(&literal);
                emit(Finding {
                    file: display_path(path),
                    literal,
                    action,
                    reason,
                })?;
                emitted += 1;
            }

            scan_tokens(path, &raw, &mut emitted, args.limit)?;
        }
    }

    Ok(())
}

fn scan_tokens(
    path: &Path,
    raw: &str,
    emitted: &mut usize,
    limit: usize,
) -> anyhow::Result<()> {
    if limit != 0 && *emitted >= limit {
        return Ok(());
    }
    let tokens: proc_macro2::TokenStream = match raw.parse() {
        Ok(t) => t,
        Err(_) => return Ok(()),
    };

    scan_token_stream(path, tokens, emitted, limit)
}

fn scan_token_stream(
    path: &Path,
    tokens: proc_macro2::TokenStream,
    emitted: &mut usize,
    limit: usize,
) -> anyhow::Result<()> {
    for tree in tokens.into_iter() {
        if limit != 0 && *emitted >= limit {
            return Ok(());
        }
        match tree {
            TokenTree::Group(group) => {
                scan_token_stream(path, group.stream(), emitted, limit)?;
            }
            TokenTree::Literal(lit) => {
                let rendered = lit.to_string();
                let Ok(lit_str) = syn::parse_str::<syn::LitStr>(&rendered) else {
                    continue;
                };
                let value = lit_str.value();
                if value.is_empty() {
                    continue;
                }
                let (action, reason) = classify_literal(&value);
                emit(Finding {
                    file: display_path(path),
                    literal: value,
                    action,
                    reason,
                })?;
                *emitted += 1;
            }
            TokenTree::Ident(_) | TokenTree::Punct(_) => {}
        }
    }
    Ok(())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn emit(finding: Finding) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string(&finding)?);
    Ok(())
}

fn classify_literal(value: &str) -> (&'static str, &'static str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return ("skip", "empty");
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return ("skip", "url");
    }
    if trimmed.contains("://") {
        return ("skip", "url_like");
    }
    if trimmed.contains('\\') || trimmed.contains('/') {
        // Heuristic: paths, regexes, code samples. Humans should review.
        return ("review", "path_or_slash");
    }
    if trimmed.contains("`") {
        return ("review", "contains_code_ticks");
    }
    if trimmed.starts_with('-') || trimmed.starts_with("--") {
        return ("skip", "flag_like");
    }
    if trimmed.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_') {
        return ("skip", "numeric_like");
    }
    ("i18n", "candidate")
}

struct LiteralVisitor {
    file_path: PathBuf,
    literals: Vec<String>,
}

impl LiteralVisitor {
    fn new(path: &Path) -> Self {
        Self {
            file_path: path.to_path_buf(),
            literals: Vec::new(),
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for LiteralVisitor {
    fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
        let value = lit.value();
        if !value.is_empty() {
            self.literals.push(value);
        }
        syn::visit::visit_lit_str(self, lit);
    }

    fn visit_file(&mut self, node: &'ast syn::File) {
        let _ = &self.file_path;
        syn::visit::visit_file(self, node);
    }
}
