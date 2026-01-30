# CODES

CODES 是一个面向终端的本地编程助手（CLI + TUI）。本仓库是基于 `openai/codex` 的 fork，并在交互、编排与工程化方面做了增强与改造。

上游链路、当前构建方式、注入链路定位等请直接看：`docs/交接文档.md`。

## Quickstart（从源码运行）

### macOS / Linux

```bash
git clone https://github.com/stevenwang288/codes.git
cd codes

# 首次运行会自动触发构建，并复用 .codes-home/ 的缓存
./codes

# 或者：带初始 prompt 启动
./codes -- "explain this codebase to me"
```

### Windows

```powershell
git clone https://github.com/stevenwang288/codes.git
cd codes

.\codes.cmd

# 或者：带初始 prompt 启动
.\codes.cmd -- "explain this codebase to me"
```

## 配置与数据目录

- 默认 Home：`~/.codes`（Windows：`C:\Users\<you>\.codes`）
- 主配置：`~/.codes/config.toml`
- 认证文件：`~/.codes/auth.json`
- 覆盖 Home：环境变量 `CODES_HOME`

## 文档入口

- 总索引：`docs/index.md`
- 安装/构建：`docs/install.md`
- 快速上手：`docs/getting-started.md`
- 配置参考：`docs/config.md`
- 斜杠命令：`docs/slash-commands.md`
- 设置面板：`docs/settings.md`
- Skills：`docs/skills.md`

## 开发与验证

- 唯一必跑校验：`./build-fast.sh`（warnings 视为失败）
- `codex-rs/` 是上游镜像（只读），实际改动写在 `code-rs/`

## 全局安装（任意目录可用 `codes`）

- Windows：`pwsh -ExecutionPolicy Bypass -File "./scripts/install-codes.ps1"`
- macOS/Linux：`./scripts/install-codes.sh`

## License

Apache-2.0（保留上游 LICENSE/NOTICE）。CODES 与 OpenAI 无任何隶属或背书关系。
