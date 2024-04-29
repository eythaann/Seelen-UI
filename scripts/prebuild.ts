import { execSync } from 'child_process';

const command = 'Get-WmiObject Win32_Process | Where-Object { $_.CommandLine -like \'*target\\debug\\*seelen*.ahk\' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }';

execSync(`powershell -ExecutionPolicy Bypass -NoProfile -Command "${command}"`);
execSync(`powershell -ExecutionPolicy Bypass -NoProfile -Command "${command}"`);