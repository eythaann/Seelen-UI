import { exec } from 'child_process';

const command = 'Get-WmiObject Win32_Process | Where-Object { $_.CommandLine -like \'*target\\debug\\*seelen*.ahk\' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }';

exec(`powershell -ExecutionPolicy Bypass -NoProfile -Command "${command}"`, (error, stdout, stderr) => {
  if (error) {
    console.error(`Error: ${error.message}`);
    return;
  }
  if (stderr) {
    console.error(`PowerShell stderr: ${stderr}`);
    return;
  }
  console.log(`PowerShell stdout: ${stdout}`);
});