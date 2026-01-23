# Purpose:
# - Apply minimal local customizations to an external opencode checkout.
# - Keep behavior stable with minimal code diffs.
#
# What this script changes:
# - Ctrl+C requires 3 presses to exit (to avoid accidental exit).
# - Installs a global AGENTS.md (Chinese-first + fixed response structure).
# - Fixes non-git project bucketing to avoid cross-folder session mixing.
# - Adds a startup "resume last session" prompt (OpenCode-native).
# - Makes zip/no-git builds more robust (script channel fallback).
#
# What this script intentionally does NOT change:
# - Claude Code compatibility: prefer using official env vars, see scripts/run-opencode.ps1.
# - System prompts/provider prompts: prefer configuration/rules over patching source.
#
# Usage:
#   pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/apply-patches.ps1" -RepoRoot "D:/path/to/opencode"

param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot
)

$ErrorActionPreference = "Stop"

function Replace-InFile([string]$path, [string]$pattern, [string]$replacement) {
  if (!(Test-Path -LiteralPath $path)) {
    throw "File not found: $path"
  }
  $content = Get-Content -Raw -LiteralPath $path
  $updated = $content -replace $pattern, $replacement
  if ($updated -ne $content) {
    Set-Content -LiteralPath $path -Value $updated -Encoding UTF8
  }
}

function Write-FileIfChanged([string]$path, [string]$content) {
  $existing = ""
  if (Test-Path -LiteralPath $path) {
    $existing = Get-Content -Raw -LiteralPath $path
  }
  if ($existing -ne $content) {
    Set-Content -LiteralPath $path -Value $content -Encoding UTF8
  }
}

function Ensure-ReplaceInFile([string]$path, [string]$pattern, [string]$replacement, [string]$guardPattern) {
  if (!(Test-Path -LiteralPath $path)) {
    throw "File not found: $path"
  }
  $content = Get-Content -Raw -LiteralPath $path
  if ($guardPattern -and $content -match $guardPattern) {
    return
  }
  $updated = $content -replace $pattern, $replacement
  if ($updated -ne $content) {
    Set-Content -LiteralPath $path -Value $updated -Encoding UTF8
  }
}

$promptPath = Join-Path $RepoRoot "packages/opencode/src/cli/cmd/tui/component/prompt/index.tsx"
$exitConfirmPath = Join-Path $RepoRoot "packages/opencode/src/cli/cmd/tui/util/exit-confirm.ts"
$appPath = Join-Path $RepoRoot "packages/opencode/src/cli/cmd/tui/app.tsx"
$sessionPath = Join-Path $RepoRoot "packages/opencode/src/cli/cmd/tui/routes/session/index.tsx"
$permissionPath = Join-Path $RepoRoot "packages/opencode/src/cli/cmd/tui/routes/session/permission.tsx"
$questionPath = Join-Path $RepoRoot "packages/opencode/src/cli/cmd/tui/routes/session/question.tsx"
$projectPath = Join-Path $RepoRoot "packages/opencode/src/project/project.ts"
$scriptIndexPath = Join-Path $RepoRoot "packages/script/src/index.ts"

# --- Project/session isolation (non-git)
# Without a .git directory, OpenCode previously used the shared "global" project bucket.
# This mixes sessions across folders and makes "resume" pick the wrong history.

$projectImportPattern = 'import fs from "fs/promises"\s*'
$projectImportReplacement = "import fs from ""fs/promises""`r`nimport { createHash } from ""crypto""`r`n"
Ensure-ReplaceInFile $projectPath $projectImportPattern $projectImportReplacement 'from "crypto"'

$nonGitBlockPattern = 'return \{\s*id: "global",\s*worktree: "\/",\s*sandbox: "\/",\s*vcs: Info\.shape\.vcs\.parse\(Flag\.OPENCODE_FAKE_VCS\),?\s*\}'
$nonGitBlockReplacement = @'
return {
        // Non-git directories previously used the shared "global" project bucket, which
        // mixes sessions across unrelated folders and breaks "resume last session".
        // Use a stable, filesystem-safe ID derived from the absolute directory.
        id: (() => {
          const abs = path.resolve(directory)
          const hash = createHash("sha256").update(abs).digest("hex").slice(0, 16)
          return "dir-" + hash
        })(),
        worktree: path.resolve(directory),
        sandbox: path.resolve(directory),
        vcs: Info.shape.vcs.parse(Flag.OPENCODE_FAKE_VCS),
      }
'@
Replace-InFile $projectPath $nonGitBlockPattern $nonGitBlockReplacement

# --- Ctrl+C exit confirm (3 presses)

$exitConfirm = @'
const EXIT_PRESS_THRESHOLD = 3
const EXIT_RESET_MS = 3000

let exitPresses = 0
let resetTimer: ReturnType<typeof setTimeout> | undefined

export function shouldExitOnCtrlC(evt: { ctrl?: boolean; name?: string }) {
  if (!evt?.ctrl || evt.name !== "c") return true

  exitPresses += 1
  if (resetTimer) clearTimeout(resetTimer)
  resetTimer = setTimeout(() => {
    exitPresses = 0
  }, EXIT_RESET_MS)

  if (exitPresses >= EXIT_PRESS_THRESHOLD) {
    exitPresses = 0
    if (resetTimer) clearTimeout(resetTimer)
    resetTimer = undefined
    return true
  }

  return false
}
'@
Write-FileIfChanged $exitConfirmPath $exitConfirm

# TUI prompt exit confirmation: import helper + gate the existing `exit()` call.
$promptImportPattern = 'import \{ useTextareaKeybindings \} from "\.\./textarea-keybindings"'
$promptImportReplacement = "import { useTextareaKeybindings } from ""../textarea-keybindings""`r`nimport { shouldExitOnCtrlC } from ""../../util/exit-confirm""`r`n"
Ensure-ReplaceInFile $promptPath $promptImportPattern $promptImportReplacement 'shouldExitOnCtrlC'
Ensure-ReplaceInFile $promptPath 'await exit\(\)' 'if (shouldExitOnCtrlC(e)) await exit()' 'shouldExitOnCtrlC\(e\)'

# App error screen Ctrl+C confirm.
$appImportPattern = 'import \{ PromptRefProvider, usePromptRef \} from "\./context/prompt"'
$appImportReplacement = "import { PromptRefProvider, usePromptRef } from ""./context/prompt""`r`nimport { shouldExitOnCtrlC } from ""./util/exit-confirm""`r`n"
Ensure-ReplaceInFile $appPath $appImportPattern $appImportReplacement 'shouldExitOnCtrlC'
$appCtrlCReplacement = @'
if (evt.ctrl && evt.name === "c") {
      if (shouldExitOnCtrlC(evt)) handleExit()
    }
'@
Replace-InFile $appPath 'if \(evt\.ctrl && evt\.name === "c"\) \{\s*handleExit\(\)\s*\}' $appCtrlCReplacement
Replace-InFile $appPath 'if \(evt\.ctrl && evt\.name === "c"\) \{`n\s*if \(shouldExitOnCtrlC\(evt\)\) handleExit\(\)`n\s*\}' $appCtrlCReplacement

# Startup resume prompt (OpenCode-native, not Claude-dependent)
$appExitImportPattern = 'import \{ shouldExitOnCtrlC \} from "\./util/exit-confirm"\s*'
$appExitImportReplacement = "import { shouldExitOnCtrlC } from ""./util/exit-confirm""`r`nimport { DialogConfirm } from ""@tui/ui/dialog-confirm""`r`n"
Ensure-ReplaceInFile $appPath $appExitImportPattern $appExitImportReplacement 'DialogConfirm'

# DialogConfirm: allow quick input 1/2 to confirm/cancel.
$dialogConfirmPath = Join-Path $RepoRoot "packages/opencode/src/cli/cmd/tui/ui/dialog-confirm.tsx"
$dialogConfirm = @'
import { TextAttributes } from "@opentui/core"
import { useTheme } from "../context/theme"
import { useDialog, type DialogContext } from "./dialog"
import { createStore } from "solid-js/store"
import { For } from "solid-js"
import { useKeyboard } from "@opentui/solid"
import { Locale } from "@/util/locale"
import { t } from "@/cli/i18n"

export type DialogConfirmProps = {
  title: string
  message: string
  onConfirm?: () => void
  onCancel?: () => void
}

export function DialogConfirm(props: DialogConfirmProps) {
  const dialog = useDialog()
  const { theme } = useTheme()
  const [store, setStore] = createStore({
    active: "confirm" as "confirm" | "cancel",
  })

  function submit(action: "confirm" | "cancel") {
    if (action === "confirm") props.onConfirm?.()
    if (action === "cancel") props.onCancel?.()
    dialog.clear()
  }

  useKeyboard((evt) => {
    if (evt.name === "return") {
      submit(store.active)
      return
    }

    // Quick-select shortcuts
    // 1 = confirm, 2 = cancel
    if (evt.name === "1" || evt.name === "y") {
      submit("confirm")
      return
    }
    if (evt.name === "2" || evt.name === "n") {
      submit("cancel")
      return
    }

    if (evt.name === "left" || evt.name === "right") {
      setStore("active", store.active === "confirm" ? "cancel" : "confirm")
    }
  })
  return (
    <box paddingLeft={2} paddingRight={2} gap={1}>
      <box flexDirection="row" justifyContent="space-between">
        <text attributes={TextAttributes.BOLD} fg={theme.text}>
          {props.title}
        </text>
        <text fg={theme.textMuted}>{t("cli.commands.tui.hint.esc")}</text>
      </box>
      <box paddingBottom={1}>
        <text fg={theme.textMuted}>{props.message}</text>
      </box>
      <box flexDirection="row" justifyContent="flex-end" paddingBottom={1}>
        <For each={["cancel", "confirm"]}>
          {(key) => (
            <box
              paddingLeft={1}
              paddingRight={1}
              backgroundColor={key === store.active ? theme.primary : undefined}
              onMouseUp={(evt) => {
                if (key === "confirm") props.onConfirm?.()
                if (key === "cancel") props.onCancel?.()
                dialog.clear()
              }}
            >
              <text fg={key === store.active ? theme.selectedListItemText : theme.textMuted}>
                {key === "confirm" ? t("cli.commands.tui.action.confirm") : t("cli.commands.tui.action.cancel")}
              </text>
            </box>
          )}
        </For>
      </box>
    </box>
  )
}

DialogConfirm.show = (dialog: DialogContext, title: string, message: string) => {
  return new Promise<boolean>((resolve) => {
    dialog.replace(
      () => (
        <DialogConfirm
          title={title}
          message={message}
          onConfirm={() => resolve(true)}
          onCancel={() => resolve(false)}
        />
      ),
      () => resolve(false),
    )
  })
}
'@
Write-FileIfChanged $dialogConfirmPath $dialogConfirm

# --- Startup resume prompt (earlier)
# Previous PatchKit versions injected a resume prompt that waited for Sync to finish,
# which made it appear noticeably late. Replace it with a lightweight, local-only
# prompt driven by Storage to show as soon as Home renders.

# Remove legacy injected block (if present)
$legacyResumePattern = '(?s)\r?\n\s*// Claude Code has a built-in "resume last" UX\.[\s\S]*?\r?\n\s*\}\)\r?\n'
Replace-InFile $appPath $legacyResumePattern "`r`n"

# Ensure imports for Storage + Instance
$storageImportPattern = 'import \{ ArgsProvider, useArgs, type Args \} from "\./context/args"\s*'
$storageImportReplacement = @'
import { ArgsProvider, useArgs, type Args } from "./context/args"
import { Storage } from "@/storage/storage"
import { Instance } from "@/project/instance"
'@
Ensure-ReplaceInFile $appPath $storageImportPattern $storageImportReplacement 'from "@/storage/storage"'

# Inject prompt after args are available
$resumeEarlyAnchor = 'const args = useArgs\(\)'
$resumeEarlyReplacement = @'
const args = useArgs()

  // PatchKit: ask to resume last session ASAP (local server only).
  // This avoids waiting for SyncProvider to complete, which can be slow.
  let patchkitResumePrompted = false
  createEffect(() => {
    if (patchkitResumePrompted) return
    if (route.data.type !== "home") return
    if (args.continue || args.sessionID || args.prompt) return

    let isLocal = false
    try {
      const u = new URL(sdk.url)
      isLocal = u.hostname === "localhost" || u.hostname === "127.0.0.1" || u.hostname === "::1"
    } catch {
      // ignore
    }
    if (!isLocal) return

    patchkitResumePrompted = true
    void (async () => {
      const cwd = process.cwd()
      const projectID = Instance.project.id
      const keys = await Storage.list(["session", projectID])

      // Avoid heavy IO on very large histories.
      const MAX_READ = 200
      const tail = keys.length > MAX_READ ? keys.slice(-MAX_READ) : keys

      let last: any | undefined
      for (const key of tail) {
        const sessionID = key[key.length - 1]
        const session = await Storage.read<any>(["session", projectID, sessionID]).catch(() => undefined)
        if (!session) continue
        if (session.parentID !== undefined) continue
        if (session.directory !== cwd) continue
        if (!last || session.time.updated > last.time.updated) {
          last = session
        }
      }

      if (!last) return
      const title = SessionApi.isDefaultTitle(last.title) ? "上次会话" : last.title
      const confirmed = await DialogConfirm.show(
        dialog,
        "继续上次会话？",
        "发现上次会话：" + title + "。按 1 继续，按 2 跳过。",
      )
      if (confirmed) {
        route.navigate({ type: "session", sessionID: last.id })
      }
    })()
  })
'@
Ensure-ReplaceInFile $appPath $resumeEarlyAnchor $resumeEarlyReplacement 'patchkitResumePrompted'

# args.continue ("-c") should also respect CWD when picking the "latest" session.
$continueCwdPattern = 'const match = sync\.data\.session\s*\r?\n\s*\.toSorted\(\(a, b\) => b\.time\.updated - a\.time\.updated\)\s*\r?\n\s*\.find\(\(x\) => x\.parentID === undefined\)\?\.id'
$continueCwdReplacement = @'
const cwd = process.cwd()
    const match = sync.data.session
      .toSorted((a, b) => b.time.updated - a.time.updated)
      .find((x) => x.parentID === undefined && x.directory === cwd)?.id
'@
Ensure-ReplaceInFile $appPath $continueCwdPattern $continueCwdReplacement 'x\.directory === cwd\)\?\.id'

# Share page: translate the generic "Thinking" label/fallback.
$webSharePartPath = Join-Path $RepoRoot "packages/web/src/components/share/part.tsx"
Replace-InFile $webSharePartPath '<span data-slot="name">Thinking</span>' '<span data-slot="name">思考中</span>'
Replace-InFile $webSharePartPath 'props\.part\.text \|\| "Thinking\.\.\."' 'props.part.text || "思考中..."'

# If the resume prompt exists, ensure it doesn't fight with the "no providers" onboarding flow.
$resumeProviderCheckPattern = '(if \(resumePrompted\) return\s*\r?\n\s*if \(sync\.status !== "complete"\) return\s*\r?\n)(\s*if \(route\.data\.type !== "home"\) return)'
$resumeProviderCheckReplacement = @'
$1    if (sync.data.provider.length === 0) return
$2
'@
Ensure-ReplaceInFile $appPath $resumeProviderCheckPattern $resumeProviderCheckReplacement 'sync\.data\.provider\.length === 0'

# --- Zip/no-git build robustness
# Some scripts try to infer a git branch name. In zip exports there is no .git, so the
# `git branch --show-current` call should not hard-fail.

$channelGitPattern = 'return await \$`git branch --show-current`\s*\.quiet\(\)\s*\.text\(\)\s*\.then\(\(x\) => x\.trim\(\)\)'
$channelGitReplacement = @'
  // Zip checkouts or exported sources may not include a .git directory.
  // Fall back to `dev` so local builds still work.
  const branch = await $`git branch --show-current`
    .quiet()
    .nothrow()
    .text()
    .then((x) => x.trim())
  return branch || "dev"
'@
Ensure-ReplaceInFile $scriptIndexPath $channelGitPattern $channelGitReplacement 'branch \|\| "dev"'

# --- Zip exports: repair broken symlink stubs
# Some zip/unpacked distributions lose symlinks and replace them with a one-line "path" file.
# This breaks TypeScript typecheck. We repair the known custom-elements.d.ts link stubs.

$uiCustomElementsPath = Join-Path $RepoRoot "packages/ui/src/custom-elements.d.ts"
if (Test-Path -LiteralPath $uiCustomElementsPath) {
  $uiCustomElementsContent = Get-Content -Raw -LiteralPath $uiCustomElementsPath
  foreach ($target in @(
    (Join-Path $RepoRoot "packages/app/src/custom-elements.d.ts"),
    (Join-Path $RepoRoot "packages/enterprise/src/custom-elements.d.ts")
  )) {
    if (!(Test-Path -LiteralPath $target)) { continue }
    $current = Get-Content -Raw -LiteralPath $target
    if ($current.Trim() -eq "../../ui/src/custom-elements.d.ts") {
      Set-Content -LiteralPath $target -Value $uiCustomElementsContent -Encoding UTF8
    }
  }
}

# Session route Ctrl+C confirm.
$sessionImportPattern = 'import \{ useKeybind \} from "@tui/context/keybind"'
$sessionImportReplacement = @'
import { useKeybind } from "@tui/context/keybind"
import { shouldExitOnCtrlC } from "../../util/exit-confirm"
'@
Ensure-ReplaceInFile $sessionPath $sessionImportPattern $sessionImportReplacement 'shouldExitOnCtrlC'
$sessionExitReplacement = @'
if (keybind.match("app_exit", evt)) {
      if (shouldExitOnCtrlC(evt)) exit()
    }
'@
Replace-InFile $sessionPath 'if \(keybind\.match\("app_exit", evt\)\) \{\s*exit\(\)\s*\}' $sessionExitReplacement
Replace-InFile $sessionPath 'if \(keybind\.match\("app_exit", evt\)\) \{`n\s*if \(shouldExitOnCtrlC\(evt\)\) exit\(\)`n\s*\}' $sessionExitReplacement

# Permission route Ctrl+C confirm.
$permissionImportPattern = 'import \{ useKeybind \} from "\.\./\.\./context/keybind"'
$permissionImportReplacement = @'
import { useKeybind } from "../../context/keybind"
import { shouldExitOnCtrlC } from "../../util/exit-confirm"
'@
Ensure-ReplaceInFile $permissionPath $permissionImportPattern $permissionImportReplacement 'shouldExitOnCtrlC'
Replace-InFile $permissionPath 'if \(evt\.name === "escape" \|\| keybind\.match\("app_exit", evt\)\)' 'if (evt.name === "escape" || (keybind.match("app_exit", evt) && shouldExitOnCtrlC(evt)))'

# Question route Ctrl+C confirm.
$questionImportPattern = 'import \{ useDialog \} from "\.\./\.\./ui/dialog"'
$questionImportReplacement = @'
import { useDialog } from "../../ui/dialog"
import { shouldExitOnCtrlC } from "../../util/exit-confirm"
'@
Ensure-ReplaceInFile $questionPath $questionImportPattern $questionImportReplacement 'shouldExitOnCtrlC'
Replace-InFile $questionPath 'if \(evt\.name === "escape" \|\| keybind\.match\("app_exit", evt\)\)' 'if (evt.name === "escape" || (keybind.match("app_exit", evt) && shouldExitOnCtrlC(evt)))'

# --- Install AGENTS.md into the OpenCode config directory

$cfgHome = if ($env:XDG_CONFIG_HOME) { $env:XDG_CONFIG_HOME } else { Join-Path $env:USERPROFILE ".config" }
$cfgDir = Join-Path $cfgHome "opencode"
New-Item -ItemType Directory -Force -Path $cfgDir | Out-Null

$scriptRoot = Split-Path -Parent $PSCommandPath
$srcAgents = Join-Path $scriptRoot "../templates/AGENTS.md"
$dstAgents = Join-Path $cfgDir "AGENTS.md"
Copy-Item -LiteralPath $srcAgents -Destination $dstAgents -Force
