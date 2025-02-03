!macro NSIS_HOOK_PREINSTALL
  ; kill the slu-service.exe process
  DetailPrint 'Exec: kill slu-service.exe'
  StrCpy $1 "wmic Path win32_process where $\"name like 'slu-service.exe' and CommandLine like '%$0%'$\" Call Terminate"
  nsExec::Exec $1
  Pop $0
  ; kill the app process
  DetailPrint 'Exec: kill seelen-ui.exe'
  StrCpy $1 "wmic Path win32_process where $\"name like 'seelen-ui.exe' and CommandLine like '%$0%'$\" Call Terminate"
  nsExec::Exec $1
  Pop $0
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Install the service
  DetailPrint 'Exec: slu-service.exe install'
  nsExec::Exec '"$INSTDIR\slu-service.exe" install'
  Pop $0
  ; Refresh file associations icons
  !insertmacro UPDATEFILEASSOC
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Gracefully stop the service
  DetailPrint 'Exec: slu-service.exe stop'
  nsExec::Exec '"$INSTDIR\slu-service.exe" stop'
  Pop $0
  ; Remove the service
  DetailPrint 'Exec: slu-service.exe uninstall'
  nsExec::Exec '"$INSTDIR\slu-service.exe" uninstall'
  Pop $0
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Refresh file associations icons
  !insertmacro UPDATEFILEASSOC
!macroend