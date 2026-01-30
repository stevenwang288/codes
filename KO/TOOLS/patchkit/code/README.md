# CODES PatchKit（本仓库本地补丁包）

目标：把你对 `code-main` 的本地改动（i18n、通知、体验增强、个人偏好配置等）集中到一个目录里，形成“可重复回放”的补丁包。

这样做的好处：

- 更新上游代码（`git pull`）后，可以一键重新应用你的本地改动。
- 避免把“私有/本地化需求”散落在仓库各处，降低合并冲突成本。
- 为后续扩展（中文-only、系统通知/声音提醒、子代理编排策略等）提供稳定落点。

## 目录结构

- `KO/TOOLS/patchkit/code/patches/`：可 `git apply` 的补丁（推荐一个 patch 一个功能点）。
- `KO/TOOLS/patchkit/code/templates/`：非代码的“模板配置”，例如 `~/.codes/config.toml` 片段。
- `KO/TOOLS/patchkit/code/scripts/`：一键脚本：导出补丁、应用补丁、更新+重放+编译。

## 快速开始（Windows / PowerShell）

本项目把所有“本地增强 / 汉化 / 自更新 / 向导工作流”都收敛到一个独立路径：

- `KO/TOOLS/patchkit/code/`

你日常只需要记住一个入口：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" help
```

1) 导出当前工作区改动为一个补丁（包含二进制 diff）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/scripts/export-patch.ps1" -RepoRoot "D:/OneDrive/steven/code/ai/12CLI/code-main"
```

2) 在“干净工作区”上应用补丁：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/scripts/apply-patches.ps1" -RepoRoot "D:/OneDrive/steven/code/ai/12CLI/code-main"
```

3) 一键：更新（ff-only）→应用补丁→编译（bash 下 `./build-fast.sh`）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/scripts/run.ps1" -RepoRoot "D:/OneDrive/steven/code/ai/12CLI/code-main" -Update -Apply -Build
```

4) 一键启动（日常用）：自动确保配置（通知钩子）+ 启动 watchdog + 启动 i18n watch + 打开 `codes` 窗口：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/scripts/start.ps1" -RepoRoot "D:/OneDrive/steven/code/ai/12CLI/code-main"
```

## 单一入口命令（推荐）

查看“我是谁/上游是谁/是否需要更新/补丁是否应用/i18n 缺口统计”：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" status -Fetch
```

向导式自更新（检查上游→询问→拉取→补丁→i18n→编译→重启）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" update -Fetch
```

只做 i18n 向导：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" i18n-wizard
```

建议在日常使用时加上 `-Configure`，确保你的 `~/.codes/config.toml` 包含通知钩子（`notify`）等配置：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/scripts/run.ps1" -RepoRoot "D:/OneDrive/steven/code/ai/12CLI/code-main" -Configure -Apply
```

## i18n 自动化闭环

你的运行时缺口采集文件默认在（归档到 PatchKit 子目录）：

- `~/.codes/patchkit/i18n-missing.jsonl`

将其自动翻译并回写到语言包（watch 模式）：

```powershell
  pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/scripts/i18n-sync.ps1" -RepoRoot "D:/OneDrive/steven/code/ai/12CLI/code-main" -Watch
```

## 子代理/长任务卡顿巡检（watchdog）

由于子代理/长任务可能会因为鉴权、网络、上下文或工具异常“看起来卡住”，PatchKit 提供一个轻量 watchdog：

- 监控 `~/.codes/history.jsonl` 与 `~/.codes/debug_logs/critical.log` 的写入活跃度
- 对应默认路径：`~/.codes/history.jsonl` 与 `~/.codes/debug_logs/critical.log`
- 如果连续一段时间没有任何新活动，会发 Windows 通知并响铃提醒你介入

启动方式：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/scripts/watchdog.ps1" -RepoRoot "D:/OneDrive/steven/code/ai/12CLI/code-main" -IntervalSeconds 60 -StallSeconds 180
```

注意：

- 只有当运行时真的遇到“zh-CN 缺 key”时才会写入 `i18n-missing.jsonl`。
- 当前仓库 `code-rs/i18n/assets/en.json` 与 `zh-CN.json` 的 key 已对齐时，不会产生缺口日志。

## 多项目复用（关键设计点）

这套机制本质上分为两层：

1) **采集层（运行时写 JSONL）**：任何项目/语言都能做，只要把缺翻译写成一行 JSON。
2) **处理层（本仓库工具）**：`KO/TOOLS/i18n-collector` 负责聚合、统计、翻译、回写。

为了适配多个项目，建议每个项目都设置统一输出到同一个 JSONL：

- `CODE_I18N_COLLECT_PATH=<somewhere>/i18n-missing.jsonl`

然后在统计/翻译时：

- `stats --group-by app` 可以按 `app` 字段区分来源项目
- 或用 `--rules <rules.json>` 自定义识别类型（CLI/TUI/站点）
