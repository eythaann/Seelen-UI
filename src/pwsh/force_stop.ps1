$runningProcesses = Get-Process | Where-Object { $_.Path -eq $ExeRoute }
if ($runningProcesses.Count -gt 0) {
  Stop-Process -Name komorebi
  Stop-Process -Name "komorebi-ui"
}