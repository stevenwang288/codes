# i18n-collector（持续式国际化缺口收集工具）

目标：你可以**正常使用 CLI/TUI**，工具自动把“缺翻译 key”落盘；等你有空时，再用 AI 批量翻译并回写到语言包。

特点：

- 不接管 stdout/stderr，不做“渲染/录屏”，避免 TUI 花屏/滑屏。
- 语言无关：任何 CLI/TUI（Rust/Go/Node/Python…）只要按 JSONL 规范写日志，都能用此工具聚合。
- 本仓库默认写入：`$CODEX_HOME/i18n-missing.jsonl`（由 `code-rs/i18n` 负责）。

## 1) JSONL 规范（最小字段）

每行一个 JSON：

```json
{"ts_ms": 0, "missing_in": "zh-CN", "key": "tui.greeting.placeholder"}
```

推荐扩展字段（跨项目更好用）：

```json
{"ts_ms": 0, "app": "code", "missing_in": "zh-CN", "key": "...", "fallback_text": "..."}
```

## 2) 使用（本仓库）

1. 正常使用（建议用 `codes` 启动），缺翻译会自动写入：

- 默认：`.codes-home/i18n-missing.jsonl`
- 可改：设置环境变量 `CODE_I18N_COLLECT_PATH=<path>`

2. 生成“待翻译任务包”（给 AI）：

```powershell
node KO/TOOLS/i18n-collector/cli.mjs extract \
  --log ".codes-home/i18n-missing.jsonl" \
  --en "code-rs/i18n/assets/en.json" \
  --out ".codes-home/i18n-todo.zh-CN.json"
```

输出文件是一个扁平 JSON：`{ "key": "English text", ... }`。

3. 自动翻译（推荐，使用 Code CLI，不改启动命令、不会花屏）：

```powershell
node KO/TOOLS/i18n-collector/cli.mjs translate \
  --in ".codes-home/i18n-todo.zh-CN.json" \
  --out ".codes-home/i18n-zh-CN.patch.json" \
  --runner coder \
  --model code-gpt-5.2-codex \
  --style zh-only
```

可选：TUI 空间足够时双语对照：`--style bilingual-tui`。

4. 回写到语言包：

```powershell
node KO/TOOLS/i18n-collector/cli.mjs apply \
  --in ".codes-home/i18n-zh-CN.patch.json" \
  --zh "code-rs/i18n/assets/zh-CN.json"
```

## 2.0) 统计（智能分类：CLI/TUI/站点）

工具会基于 key 前缀（例如 `cli.*` / `tui.*` / `docs.*`/`site.*`）和 `app` 字段做启发式分类，并支持用规则文件覆盖。

```powershell
node KO/TOOLS/i18n-collector/cli.mjs stats \
  --log ".codes-home/i18n-missing.jsonl" \
  --en "code-rs/i18n/assets/en.json" \
  --zh "code-rs/i18n/assets/zh-CN.json" \
  --group-by type
```

按 app 维度查看（跨项目汇总时有用）：

```powershell
node KO/TOOLS/i18n-collector/cli.mjs stats \
  --log ".codes-home/i18n-missing.jsonl" \
  --en "code-rs/i18n/assets/en.json" \
  --zh "code-rs/i18n/assets/zh-CN.json" \
  --group-by app \
  --top 20
```

可选：用规则文件覆盖分类（适配多个项目）：

```json
{
  "rules": [
    { "type": "tui", "key_prefixes": ["tui."] },
    { "type": "cli", "key_prefixes": ["cli."] },
    { "type": "site", "key_prefixes": ["docs.", "site.", "web."] },
    { "type": "tui", "app_regex": "(code-tui|.*tui.*)" }
  ]
}
```

```powershell
node KO/TOOLS/i18n-collector/cli.mjs stats \
  --log ".codes-home/i18n-missing.jsonl" \
  --en "code-rs/i18n/assets/en.json" \
  --zh "code-rs/i18n/assets/zh-CN.json" \
  --rules ".codes-home/i18n.rules.json" \
  --group-by type
```

## 2.1) 一键闭环（sync）

将 `extract -> translate -> apply` 串起来：

```powershell
node KO/TOOLS/i18n-collector/cli.mjs sync \
  --log ".codes-home/i18n-missing.jsonl" \
  --en "code-rs/i18n/assets/en.json" \
  --zh "code-rs/i18n/assets/zh-CN.json" \
  --runner coder \
  --model code-gpt-5.2-codex \
  --style zh-only \
  --once
```

持续监听（缺口日志追加时自动回写）：

```powershell
node KO/TOOLS/i18n-collector/cli.mjs sync \
  --log ".codes-home/i18n-missing.jsonl" \
  --en "code-rs/i18n/assets/en.json" \
  --zh "code-rs/i18n/assets/zh-CN.json" \
  --runner coder \
  --model code-gpt-5.2-codex \
  --style zh-only \
  --watch
```

## 3) 使用（其它项目）

只要其它项目也把缺口写到同样的 JSONL（append 模式），本工具同样可以 `extract/apply`。

建议每个项目设置：

- `CODE_I18N_COLLECT_PATH=<somewhere>/i18n-missing.jsonl`

这样你不用改启动命令也能持续积累（例如写到系统环境变量或全局 profile）。
