# 万能补丁机制（PatchKit / CODES）

目标：**换一台机器/换一个环境**，只要拉下源码，再跑一遍 PatchKit 流程，就能把我们对 CODES 的所有改造**稳定、可重复**地复现出来，并能在上游更新后**重新回放**这些改造。

## 1) PatchKit 是什么（在本项目里的定义）

这里的“补丁”不是单指某个 `.patch` 文件，而是一套**可回放的工程化流程**：

- 可选：检查/拉取上游更新
- 应用一组可重放的补丁（patch files）
- 可选：i18n 缺口闭环（运行时采集 → 翻译 → 回写）
- 可选：编译与启动
- 可选：watchdog 监控“无增量卡住”，提供桌面通知/声音提醒

PatchKit 的价值：不再靠“人脑记住我改过哪些文件”，而是让这些变更**可脚本化、可迁移、可回滚、可验收**。

## 2) 入口与目录约定（单一入口）

- PatchKit 入口：`KO/TOOLS/patchkit/code/patchkit.ps1`
- Patch 文件目录：`KO/TOOLS/patchkit/code/patches/*.patch`
- 运行时目录：CODES 默认 `~/.codes/`（可用 `CODES_HOME` 覆盖）
  - i18n 缺口日志默认：`~/.codes/patchkit/i18n-missing.jsonl`

## 3) 新机器上的最短流程（Windows / PowerShell）

1) 状态检查（推荐第一步）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" status -Fetch
```

2) 向导式更新（可选 pull/apply/i18n/build/restart）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" update -Fetch
```

3) 只应用补丁：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" apply
```

4) 只确保配置（通知/提示钩子等写入 `~/.codes/config.toml`）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" config
```

5) 启动（默认会启动 watchdog + i18n watch + 打开 `codes` 窗口）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/patchkit.ps1" start
```

## 4) 如何把“当前工作区改动”变成可回放补丁

PatchKit 提供导出脚本：

```powershell
pwsh -ExecutionPolicy Bypass -File "./KO/TOOLS/patchkit/code/scripts/export-patch.ps1" -RepoRoot "." -Name "my-change"
```

导出的 `.patch` 会写到：`KO/TOOLS/patchkit/code/patches/`。

约定：**一个 patch 对应一个功能点**，降低冲突与回滚成本。

## 5) 与 CODES 的独立性关系（重要）

- CODES 运行时默认只使用 `~/.codes/`，不与 `~/.codex` / `~/.code` 共享状态。
- PatchKit 只负责“把改造稳定注入到源码 + 提供辅助工作流”，不改变上述独立性原则。

