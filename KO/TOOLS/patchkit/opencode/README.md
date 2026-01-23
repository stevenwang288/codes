# OpenCode PatchKit（code-main 工具）

PatchKit 是给你本地的 `opencode` 源码 checkout 打补丁的“可重复应用”工具集。

## 放在哪里、叫什么

- 位置：`code-main/tools/opencode-patchkit`
- 名称：继续叫 `opencode-patchkit`（语义明确：对 opencode 做 patch 的工具包；同时和 `tools/i18n-collector` 的命名风格一致）

把它放在 `tools/` 的原因：

- 不把个人定制混进上游 opencode 仓库，更新上游时更干净
- 仍然能一键对任意 opencode checkout 重打补丁

## 配置 opencode RepoRoot（建议做一次）

PatchKit 需要知道你的 opencode 源码目录（即包含 `packages/opencode/package.json` 的目录）。任选一种方式：

1) 环境变量：`OPENCODE_REPO_ROOT=D:/path/to/opencode`
2) 配置文件：复制 `patchkit.json.example` 为 `patchkit.json` 并修改 `opencodeRepoRoot`
3) 不配置：PatchKit 会尝试自动发现（同级 `../opencode/opencode-dev` 等），但不建议长期依赖

## 这个 PatchKit 改了什么（最小改动）

1) 强制中文 + 固定回复结构
   - 覆盖安装全局：`~/.config/opencode/AGENTS.md`（来源：`templates/AGENTS.md`）

2) 禁用 Claude Code 兼容层注入（不改源码）
   - 运行时注入 env：`OPENCODE_DISABLE_CLAUDE_CODE=1`（同时显式禁 prompt/skills）

3) TUI 防误退：`Ctrl+C` 需要按 3 次才退出

4) 非 git 目录 projectID 分桶修复
   - 避免不同目录会话混桶导致“恢复到错项目”

5) 启动时提示“继续上次会话？”（更早出现）
   - 新策略：仅本地 server（localhost/127.0.0.1）直接从 `Storage` 扫描当前目录会话并提示
   - 目的：不再等待 `SyncProvider` 完成导致弹窗“出现太晚”

6) zip/no-git 构建稳健性 + 已知 symlink stub 修复（可选但建议）

## 用法

只打补丁：

```powershell
pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/run.ps1" -RepoRoot "D:/path/to/opencode"
```

日常推荐入口（快）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/start.ps1" -RepoRoot "D:/path/to/opencode"
```

说明：`start.ps1` 默认走“智能 build”（仅当编译产物不存在或关键输入变化时才重编译），并且默认不跑 `cleanup-legacy`，启动链路更短。

全局命令（Windows）：安装后 `op`、`OP`、`opencode` 都可用（PatchKit 会生成对应的 `.cmd` 启动器）。

强制重编译：

```powershell
pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/start.ps1" -RepoRoot "D:/path/to/opencode" -Build
```

清理旧 shim（偏慢，默认不跑）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/start.ps1" -RepoRoot "D:/path/to/opencode" -CleanupLegacy
```

向导模式（分步执行，每步询问是否继续）：

```powershell
pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/wizard.ps1" -RepoRoot "D:/path/to/opencode"
```

## 维护提示

- `scripts/apply-patches.ps1` 使用 regex 替换；上游若大改文件结构可能需要更新匹配模式。
- PatchKit 会记录 build-stamp（`tools/opencode-patchkit/.state/build-stamps.json`）用于跳过不必要的重编译。
