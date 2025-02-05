!macro NSIS_HOOK_PREINSTALL
  StrCpy $1 "taskkill.exe /F /T /IM slu-service.exe"
  DetailPrint 'Exec: $1'
  nsExec::Exec $1
  Pop $0
  StrCpy $1 "taskkill.exe /F /T /IM seelen-ui.exe"
  DetailPrint 'Exec: $1'
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