!macro NSIS_HOOK_PREINSTALL
  ; add code here
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Refresh file associations icons
  !insertmacro UPDATEFILEASSOC
  ; Create the app service
  FILE "${__FILEDIR__}\..\..\slu-service.exe"
  nsExec::Exec '"$INSTDIR\slu-service.exe" install'
  Pop $0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; add code here
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Refresh file associations icons
  !insertmacro UPDATEFILEASSOC
  ; Delete the app service
  nsExec::Exec '"$INSTDIR\slu-service.exe" uninstall'
  Pop $0
  Delete "$INSTDIR\slu-service.exe"
!macroend