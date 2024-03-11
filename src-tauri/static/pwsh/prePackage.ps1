param (
  [string]$folderPath
)

$runningProcesses = Get-Process | Where-Object { $_.Path -like "$folderPath\*" }
if ($runningProcesses.Count -gt 0) {
    $runningProcesses | ForEach-Object { Stop-Process -Id $_.Id -Force }
}