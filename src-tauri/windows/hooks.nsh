!macro FREEDPI_STOP_ENGINE
  nsExec::Exec 'taskkill /F /IM goodbyedpi.exe'
  nsExec::Exec 'sc stop WinDivert1.4'
  nsExec::Exec 'sc stop WinDivert'
  Sleep 1500
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro FREEDPI_STOP_ENGINE
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec 'schtasks /Delete /TN FreeDPI /F'
  !insertmacro FREEDPI_STOP_ENGINE
!macroend
