param(
  [Parameter(Mandatory = $true, Position = 0)]
  [string]$NotificationJson
)

Set-StrictMode -Version Latest

function Try-PlaySound {
  param([ValidateSet('Asterisk','Beep','Exclamation','Hand','Question')][string]$Kind = 'Asterisk')

  try {
    Add-Type -AssemblyName System.Media -ErrorAction SilentlyContinue | Out-Null
    $sounds = [System.Media.SystemSounds]
    switch ($Kind) {
      'Beep' { $sounds::Beep.Play() }
      'Exclamation' { $sounds::Exclamation.Play() }
      'Hand' { $sounds::Hand.Play() }
      'Question' { $sounds::Question.Play() }
      default { $sounds::Asterisk.Play() }
    }
  } catch {
    # Ignore sound errors.
  }
}

function Send-ToastBurntToast {
  param([string]$Title, [string]$Message)
  try {
    if (-not (Get-Module -ListAvailable -Name BurntToast)) {
      return $false
    }
    Import-Module BurntToast -ErrorAction Stop
    New-BurntToastNotification -Text $Title, $Message | Out-Null
    return $true
  } catch {
    return $false
  }
}

function Send-BalloonFallback {
  param([string]$Title, [string]$Message)
  try {
    Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop
    Add-Type -AssemblyName System.Drawing -ErrorAction Stop

    $icon = New-Object System.Windows.Forms.NotifyIcon
    $icon.Icon = [System.Drawing.SystemIcons]::Information
    $icon.BalloonTipTitle = $Title
    $icon.BalloonTipText = $Message
    $icon.Visible = $true
    $icon.ShowBalloonTip(4000)

    Start-Sleep -Milliseconds 500
    $icon.Dispose()
    return $true
  } catch {
    return $false
  }
}

try {
  $notification = $NotificationJson | ConvertFrom-Json
} catch {
  # If parsing fails, still notify in a minimal way.
  $title = "CODES"
  $msg = "notify: invalid json"
  [void](Send-ToastBurntToast -Title $title -Message $msg)
  [void](Send-BalloonFallback -Title $title -Message $msg)
  Try-PlaySound -Kind 'Exclamation'
  exit 0
}

$type = [string]($notification.type ?? $notification.event ?? 'unknown')
$title = "CODES"
$message = ""

switch ($type) {
  'agent-turn-complete' {
    $title = "CODES: 回合完成"
    $last = [string]($notification.'last-assistant-message')
    if ($last) {
      $title = "CODES: $last"
    }
    $inputs = @($notification.input_messages)
    if ($inputs.Count -gt 0) {
      $message = ($inputs -join ' ')
    } else {
      $message = "turn complete"
    }
    Try-PlaySound -Kind 'Asterisk'
  }
  'i18n-sync' {
    $title = "CODES: i18n 回写完成"
    $inputs = @($notification.input_messages)
    $message = ($inputs -join ' ')
    if (-not $message) { $message = "i18n sync" }
    Try-PlaySound -Kind 'Asterisk'
  }
  'watchdog-stall' {
    $title = "CODES: 可能卡住"
    $inputs = @($notification.input_messages)
    $message = ($inputs -join ' ')
    if (-not $message) { $message = "stall" }
    Try-PlaySound -Kind 'Exclamation'
  }
  'approval-requested' {
    $title = "CODES: 需要审批"
    $message = "approval requested"
    Try-PlaySound -Kind 'Exclamation'
  }
  default {
    $title = "CODES: $type"
    $message = "event"
    Try-PlaySound -Kind 'Question'
  }
}

if (-not $message) { $message = "(no message)" }

if (-not (Send-ToastBurntToast -Title $title -Message $message)) {
  [void](Send-BalloonFallback -Title $title -Message $message)
}

exit 0
