$taskName = "KomorebiUI"
$command = "$PSScriptRoot\komorebi.exe -c $Env:USERPROFILE\.config\komorebi\settings.json"

$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-WindowStyle Hidden -Command $command"
$trigger = New-ScheduledTaskTrigger -AtLogon
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -Hidden

$existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
if ($existingTask -ne $null) {
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
}

Register-ScheduledTask -Action $action -Trigger $trigger -Settings $settings -TaskName $taskName -User $env:USERNAME -RunLevel Highest