!macro NSIS_HOOK_PREINSTALL
  ; add code here
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Refresh file associations icons
  !insertmacro UPDATEFILEASSOC
  ; Create the app service
  ExecWait '"$INSTDIR\slu-service.exe" install'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; add code here
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Refresh file associations icons
  !insertmacro UPDATEFILEASSOC
  ; Delete the app service
  ExecWait '"$INSTDIR\slu-service.exe" uninstall'
!macroend