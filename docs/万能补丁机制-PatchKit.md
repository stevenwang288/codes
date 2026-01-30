# 万能补丁机制（PatchKit）

你要的目标是：**换一台机器/换一个环境**，只要把源代码拉下来，再跑一遍“补丁流程”（最好脚本驱动，必要时可由 AI 驱动），就能把这几天的所有改造**稳定、可重复**地复现出来。

在本仓库里，这套机制已经有一个天然落点：`KO/TOOLS/patchkit/`。

- `KO/TOOLS/patchkit/code/`：面向本仓库 `code-main` 的“万能补丁包”（本文以它为准）
- `KO/TOOLS/patchkit/opencode/`：面向 OpenCode 的补丁包（另一个项目）
- `patches/`（仓库根目录）：Bazel/构建相关补丁（与 PatchKit 的“产品补丁”不同层，别混用）

本文是“**实施型文档**”：不做空泛说明，直接给出

1) 机制设计（为什么这样组织）
2) 在新机器上的**可执行流程**
3) 本轮已经修改过的**具体文件清单**（文件 → 改了什么 → 属于哪个补丁点）

---

## 0. 一句话定义“补丁”

在这里，“补丁”不是单指某一个 `.patch` 文件，而是一个**可回放的工程化流程**：

- **对上游版本**做检查/更新（可选）
- 对工作区应用一组**可重复回放**的变更（patches + i18n + 配置注入）
- 最后执行一次**编译/自测/启动**（你要求最终要能跑起来）

PatchKit 的价值在于：你不用记“我改过哪些文件”，也不用在升级后手工合并；它把这些动作机械化。

---

## 1. PatchKit 的“单一入口”与目录约定

### 1.1 入口脚本

- 单入口：`KO/TOOLS/patchkit/code/patchkit.ps1`

它封装了日常会用到的动作：

- `status`：查看上游/本地/补丁应用状态、i18n 缺口统计
- `update`：向导式更新（抓上游 → 拉取 → 应用补丁 → i18n →（可选）编译/重启）
- `config`：确保 `~/.codex/config.toml` 写入通知钩子与 TUI 通知配置
- `apply`：应用 `KO/TOOLS/patchkit/code/patches/*.patch`
- `start`：启动 codes + watchdog + i18n watch

### 1.2 补丁文件存放

- 代码/文档/资源的可回放补丁：`KO/TOOLS/patchkit/code/patches/*.patch`
  - 约定：**一个 patch 对应一个功能点**，降低冲突与回滚成本

### 1.3 配置与状态目录（你要求的核心）

你明确要求：**CODES 与 CODEX 共用同一套配置/状态目录**，统一以 `CODEX_HOME`/`~/.codex` 为准。

这轮我已把 PatchKit 的默认目录从历史的 `.codes` 纠正到 `.codex`：

- 默认 `CODEX_HOME = ~/.codex`
- i18n 缺口日志：`${CODEX_HOME}/patchkit/i18n-missing.jsonl`

对应实现文件见第 3 节清单。

---

## 2. 新机器上的“可回放流程”（脚本驱动）

> 这部分就是你要的“换环境也能一键复现”的机械流程。

### 2.1 前置依赖

- Windows：PowerShell 7（`pwsh`）
- Node.js（用于 i18n collector）
- Git
- bash（Git for Windows 自带 bash 即可，用于跑 `./build-fast.sh`）

### 2.2 拉取源码

```powershell
git clone <你的仓库地址> code-main
cd code-main
```

### 2.3 （可选）配置 PatchKit 上游 remote/branch

- 默认会选 `upstream` 或 `origin`。
- 如果你想强制：复制示例配置并改：

`KO/TOOLS/patchkit/code/patchkit.json.example` → `KO/TOOLS/patchkit/code/patchkit.json`

### 2.4 一键状态检查（推荐第一步）

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" status -Fetch
```

### 2.5 一键“更新 + 回放 +（可选）编译 + 重启”

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" update -Fetch
```

### 2.6 只应用补丁（不更新）

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" apply
```

### 2.7 只确保配置（不动代码）

把通知、TUI 通知开关等写入 `~/.codex/config.toml`：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" config
```

### 2.8 启动（附带 watchdog + i18n watch）

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" start -StartWatchdog -I18nWatch
```

---

## 3. 本轮“实际改了什么”（文件级清单）

> 这是你强调的核心：不是“说明”，而是“落到文件”。
> 由于当前环境无法执行 `git diff`，这里按我们已落盘的修改逐条列出；后续建议用 PatchKit 的 `export-patch.ps1` 把这些变更导出为 patch 文件，纳入可回放体系。

### 3.1 配置目录统一到 `~/.codex`（CODEX_HOME）

- `code-rs/core/src/config/sources.rs`
  - `find_code_home()`：只支持 `CODEX_HOME` 或默认 `~/.codex`，不再回退旧目录（你要求“CODE 目录废弃”）。

- `code-rs/core/src/config.rs`
  - 默认 `file_opener` 设为 `None`（你反馈“路径打开/改写”行为反而伤害使用，先彻底关掉）。

### 3.2 注入规则：以 `user_instructions` 为单一权威来源

- `code-rs/core/src/project_doc.rs`
  - 当 `config.user_instructions` 非空时：
    - 不再扫描/拼接工作区内的 `AGENTS.md`
    - Auto Drive 的 `AUTO_AGENTS.md` 也不再注入
  - 目的：你说了规则就必须生效，不允许被项目内 AGENTS 覆盖。

### 3.3 TUI：设置页左键/ESC 回退问题

- `code-rs/tui/src/chatwidget/settings_handlers.rs`
  - 菜单（子视图）激活时：`Left/Esc` 先退出菜单回到左侧列表，而不是直接关闭设置页。

### 3.4 TUI：主题列表从“很多”退化到“明/暗”

- `code-rs/tui/src/theme.rs`
  - Windows 颜色模式强制走 `Ansi256`，避免退回 `Ansi16` 导致主题列表只剩两项。

### 3.5 i18n：补齐设置左侧列表/提示/占位的 key（解决你截图那一列的英文）

- `code-rs/i18n/assets/zh-CN.json`
  - 新增 `tui.settings.section.* / tui.settings.help.* / tui.settings.placeholder.*`

> 注意：你还要求“每一项进去看子/孙菜单英文残留”，这部分属于下一轮的定点补齐（不做全量扫），应继续在 `zh-CN.json` 增 key 并替换各 view 的硬编码文案。

### 3.6 启动脚本：`codes` 报 “. was unexpected at this time.”

- `codes.cmd`
  - 修正 `%~dp0` 路径处理：无条件去掉末尾反斜杠，避免 `if`/字符串切片触发批处理解析异常。

### 3.7 文档与需求记录（交付/对齐用）

- `docs/交接文档.md`
  - 记录当前已做与未做、以及“执行通道坏了暂时无法编译”的事实（便于交接）。

- `docs/需求记录.md`
  - 持久化记录：鼠标选中自动复制、通知/声音提醒、OpenCode 对照调研等需求。

- `docs/*.md` 与 `code-rs/config.md`
  - 多处把旧的 `.code`/`~/.code` 路径文案改为 `.codex`/`~/.codex`（与你的统一策略对齐）。

### 3.8 PatchKit 自身：把旧 `.codes` 体系升级到 `.codex`

- `KO/TOOLS/patchkit/code/patchkit.json.example`
  - 默认 `codeHome` 改为 `.codex`
  - 默认 i18n log 改为 `patchkit/i18n-missing.jsonl`

- `KO/TOOLS/patchkit/code/scripts/config.ps1`
  - `Resolve-CodeHome`：优先 `CODEX_HOME`，否则默认 `~/.codex`
  - `Resolve-I18nLogPath`：默认 `${CODEX_HOME}/patchkit/i18n-missing.jsonl`

- `KO/TOOLS/patchkit/code/scripts/_lib.ps1`
  - `Invoke-Bash`：不再把 `CODEX_HOME` 强制指到仓库内 `.codes-home`，改为对齐 `${HOME}/.codex`

- `KO/TOOLS/patchkit/code/patchkit.ps1`
  - `Print-TestInstructions`：改为使用 `Resolve-CodeHome/Resolve-I18nLogPath`，不再硬编码 `.codes`

- `KO/TOOLS/patchkit/code/README.md`
  - 更新所有路径示例：统一到 `KO/TOOLS/patchkit/code/` 与 `~/.codex`

---

## 4. 你要的“万能补丁”下一步怎么落地（最短路径）

> 你说的“最好脚本驱动补丁”不是从 0 发明：PatchKit 已经能做 80%。剩下 20% 是把零散改动都收进 `patches/*.patch`，形成可重放。

### 4.1 把当前工作区改动导出为补丁文件

- 脚本：`KO/TOOLS/patchkit/code/scripts/export-patch.ps1`
- 目标：把本轮对 Rust/i18n/docs 的改动，导出为 1~N 个 patch 文件放到 `KO/TOOLS/patchkit/code/patches/`。

建议命名：

- `0001-codex-home-unify.patch`
- `0002-settings-left-back.patch`
- `0003-settings-i18n-sidebar.patch`
- `0004-disable-file-opener.patch`
- `0005-patchkit-codex-home.patch`

### 4.2 以后升级上游的固定动作

1) `patchkit status -Fetch`
2) `patchkit update -Fetch`（或手动 pull + apply）
3) 只在最后跑一次 `./build-fast.sh`

---

## 5. “过去式文档”如何处理

你提到这几天写了很多文档，部分已经过时（例如仍提 `.codes-home` / `CODE_HOME` 的旧闭环）。

建议策略：

- **不删**（保留历史线索）
- 新增本文作为“唯一权威入口”
- 后续所有变更都以“PatchKit 可回放”为准，避免再产生新的散文式文档

---

## 6. 需要你确认的一件事（只问一次）

你要的“万能补丁”落盘目录，是否就以 `KO/TOOLS/patchkit/code/` 为唯一入口？

- 如果是：后续所有改动我都会优先输出为 `KO/TOOLS/patchkit/code/patches/*.patch` + 必要脚本更新。
- 如果你想换目录（例如你说的 `KO` 以外的某个新目录），我再做一次迁移。