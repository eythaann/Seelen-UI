param (
  [string]$ExeRoute
)

$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    $ownRoute = $MyInvocation.MyCommand.Definition
    Start-Process powershell -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$ownRoute`" -ExeRoute `"$ExeRoute`"" -Verb RunAs -WindowStyle Hidden
    Exit
}

$taskName = "KomorebiUI"
$jsonPath = Join-Path $Env:USERPROFILE "\.config\komorebi-ui\settings.json"
$jsonContent = Get-Content -Raw -Path $jsonPath | ConvertFrom-Json

$actions = @(New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -WindowStyle Hidden -Command `"& '$ExeRoute' -c $jsonPath`"")

if ($jsonContent.ahk_enabled -eq $true) {
  $ahkPath = Join-Path $Env:USERPROFILE "\.config\komorebi-ui\komorebic.ahk"
  $actions += New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -WindowStyle Hidden -Command `"& '$ahkPath'`""
}

$trigger = New-ScheduledTaskTrigger -AtLogon
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -Hidden

$existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
if ($null -ne $existingTask) {
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
}

Register-ScheduledTask -Action $actions -Trigger $trigger -Settings $settings -TaskName $taskName -User $env:USERNAME -RunLevel Highest