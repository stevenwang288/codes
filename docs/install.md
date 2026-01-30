## Installing & building

### System requirements

| Requirement                 | Details                                                         |
| --------------------------- | --------------------------------------------------------------- |
| Operating systems           | macOS 12+, Ubuntu 20.04+/Debian 10+, or Windows 11 (native; Git Bash recommended) |
| Git (optional, recommended) | 2.23+ for built-in PR helpers                                   |
| RAM                         | 4-GB minimum (8-GB recommended)                                 |

### Build from source

```bash
# Clone the repository and navigate to the workspace root.
git clone https://github.com/stevenwang288/codes.git
cd codes

# Install the Rust toolchain, if necessary.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Build everything (CLI, TUI, MCP servers). This is the same check CI runs.
./build-fast.sh

# Launch the TUI with a sample prompt.
./codes -- "explain this codebase to me"
```

> [!NOTE]
> The project treats compiler warnings as errors. The only required local check is `./build-fast.sh`; skip `rustfmt`/`clippy` unless asked.

### Install globally (so `codes` works anywhere)

This installs `codes` into `~/.codes/bin` and makes it available on your `PATH`.

#### Windows (PowerShell)

```powershell
pwsh -ExecutionPolicy Bypass -File "./scripts/install-codes.ps1"

# Restart your terminal, then:
codes --version
```

> If you already built and only want to copy binaries: add `-NoBuild`.

#### macOS / Linux

```bash
./scripts/install-codes.sh
codes --version
```
