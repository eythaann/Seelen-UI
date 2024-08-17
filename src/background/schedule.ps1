param (
  [string]$ExeRoute,
  [string]$Enabled
)

$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  $ownRoute = $MyInvocation.MyCommand.Definition
  $arguments = @(
    "-NoProfile"
    "-ExecutionPolicy Bypass"
    "-File `"$ownRoute`""
    "-ExeRoute `"$ExeRoute`""
    "-Enabled `"$Enabled`""
  )
  Start-Process powershell -ArgumentList $arguments -Verb RunAs -WindowStyle Hidden
  Exit
}

$taskName = "\Seelen\Seelen-UI"

if ($Enabled -eq "true") {
  $action = New-ScheduledTaskAction -Execute "$ExeRoute" -Argument "--silent"
  $trigger = New-ScheduledTaskTrigger -AtLogon
  $settings = New-ScheduledTaskSettingsSet -Priority 2 -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -Hidden

  Register-ScheduledTask -Force -Action $action -Trigger $trigger -Settings $settings -TaskName $taskName -User $env:USERNAME -RunLevel Highest
}
else {
  $existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
  if ($null -ne $existingTask) {
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
  }
}
