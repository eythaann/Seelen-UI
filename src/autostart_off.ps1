$taskName = "KomorebiUI"
$existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
if ($existingTask -ne $null) {
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
}