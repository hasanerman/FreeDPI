!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec 'schtasks /Delete /TN FreeDPI /F'
!macroend
