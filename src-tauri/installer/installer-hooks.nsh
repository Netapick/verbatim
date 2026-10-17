; Installateur léger : les moteurs et accélérations (dossier « redist ») sont téléchargés pendant l'installation.
; Les tailles, sommes de contrôle et l'URL viennent de packs.nsh (généré par scripts/make-packs.ps1).
!include "LogicLib.nsh"
!define VB_DIR "${__FILEDIR__}"
!include "${VB_DIR}\packs.nsh"

; Journal de l'installation des composants : $INSTDIR\redist-install.log
!macro VB_LOG MSG
  FileOpen $9 "$INSTDIR\redist-install.log" a
  FileSeek $9 0 END
  FileWrite $9 "${MSG}$\r$\n"
  FileClose $9
!macroend

!macro VB_FETCH LABEL FILE SHA MB
  DetailPrint "${LABEL} (environ ${MB} Mo)…"
  !insertmacro VB_LOG "[hook] ${LABEL}"
  InitPluginsDir
  File "/oname=$PLUGINSDIR\dl-pack.ps1" "${VB_DIR}\dl-pack.ps1"
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "$PLUGINSDIR\dl-pack.ps1" -Url "${PACKS_BASE}/${FILE}" -Sha "${SHA}" -Dest "$INSTDIR" -SizeMb ${MB}'
  Pop $0
  !insertmacro VB_LOG "[hook] code retour : $0"
  ${If} $0 != "0"
    DetailPrint "Échec : ${LABEL}."
    ${IfNot} ${Silent}
      MessageBox MB_OK|MB_ICONEXCLAMATION "Le téléchargement de « ${LABEL} » a échoué.$\r$\n$\r$\nVérifiez votre connexion Internet. Verbatim fonctionnera, mais sans ces composants : vous pourrez relancer l'installateur pour les ajouter."
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro VB_LOG "[hook] POSTINSTALL"
  ${IfNot} ${FileExists} "$INSTDIR\redist\engine\*.*"
    ${If} ${Silent}
      !insertmacro VB_FETCH "Moteurs de transcription et d'IA" "${PACK_CORE_FILE}" "${PACK_CORE_SHA}" "${PACK_CORE_MB}"
    ${Else}
      MessageBox MB_YESNO|MB_ICONQUESTION "Verbatim a besoin de ses moteurs de transcription (Whisper) et d'intelligence artificielle (llama.cpp).$\r$\n$\r$\nTéléchargement : environ ${PACK_CORE_MB} Mo.$\r$\n$\r$\nLes installer maintenant ?" IDNO vb_skip_core
      !insertmacro VB_FETCH "Moteurs de transcription et d'IA" "${PACK_CORE_FILE}" "${PACK_CORE_SHA}" "${PACK_CORE_MB}"
      vb_skip_core:
    ${EndIf}
  ${EndIf}
  ${IfNot} ${Silent}
  ${AndIfNot} ${FileExists} "$INSTDIR\redist\engine-rocm\*.*"
    MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 "Accélération par carte graphique AMD (ROCm).$\r$\n$\r$\nRéservée aux cartes Radeon RX 7000 (RDNA 3) : transcription et rédaction beaucoup plus rapides.$\r$\nTéléchargement : environ ${PACK_ROCM_MB} Mo.$\r$\n$\r$\nL'installer maintenant ?" IDNO vb_skip_rocm
    !insertmacro VB_FETCH "Accélération AMD (ROCm)" "${PACK_ROCM_FILE}" "${PACK_ROCM_SHA}" "${PACK_ROCM_MB}"
    vb_skip_rocm:
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  RMDir /r "$INSTDIR\redist"
  ${IfNot} ${Silent}
    ${If} ${FileExists} "$INSTDIR\ia\*.*"
      MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 "Supprimer aussi les modèles téléchargés (dossier « ia », plusieurs Go) ?" IDNO vb_keep_models
      RMDir /r "$INSTDIR\ia"
      vb_keep_models:
    ${EndIf}
  ${EndIf}
!macroend
