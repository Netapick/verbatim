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

; Moteur Whisper CUDA officiel (NVIDIA), extrait à plat dans ia\engine-gpu : même source que le bouton des Réglages.
!define CUDA_URL "https://github.com/ggml-org/whisper.cpp/releases/download/b5454/whisper-bin-win-cuda-12.4.0-x64.zip"
!define CUDA_MB "685"

!macro VB_FETCH_URL LABEL URL MB DEST
  DetailPrint "${LABEL} (environ ${MB} Mo)…"
  !insertmacro VB_LOG "[hook] ${LABEL}"
  InitPluginsDir
  File "/oname=$PLUGINSDIR\dl-pack.ps1" "${VB_DIR}\dl-pack.ps1"
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "$PLUGINSDIR\dl-pack.ps1" -Url "${URL}" -Dest "${DEST}" -SizeMb ${MB} -Log "$INSTDIR\redist-install.log" -Flat'
  Pop $0
  !insertmacro VB_LOG "[hook] code retour : $0"
  ${If} $0 != "0"
    DetailPrint "Échec : ${LABEL}."
    ${IfNot} ${Silent}
      MessageBox MB_OK|MB_ICONEXCLAMATION "Le téléchargement de « ${LABEL} » a échoué.$\r$\n$\r$\nVerbatim fonctionnera sur le processeur : vous pourrez réessayer depuis les Réglages."
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro VB_LOG "[hook] POSTINSTALL"
  ${IfNot} ${FileExists} "$INSTDIR\redist\engine\*.*"
    !insertmacro VB_FETCH "Moteurs de transcription et d'IA" "${PACK_CORE_FILE}" "${PACK_CORE_SHA}" "${PACK_CORE_MB}"
  ${EndIf}
  ; Accélération GPU : détectée puis installée directement, sans question.
  InitPluginsDir
  File "/oname=$PLUGINSDIR\detect-gpu.ps1" "${VB_DIR}\detect-gpu.ps1"
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "$PLUGINSDIR\detect-gpu.ps1"'
  Pop $2
  !insertmacro VB_LOG "[hook] détection GPU, code : $2"
  IntOp $1 $2 & 2
  ${If} $1 != 0
  ${AndIfNot} ${FileExists} "$INSTDIR\redist\engine-rocm\*.*"
    !insertmacro VB_FETCH "Accélération AMD (ROCm)" "${PACK_ROCM_FILE}" "${PACK_ROCM_SHA}" "${PACK_ROCM_MB}"
  ${EndIf}
  IntOp $1 $2 & 1
  ${If} $1 != 0
  ${AndIfNot} ${FileExists} "$INSTDIR\ia\engine-gpu\whisper-cli.exe"
    CreateDirectory "$INSTDIR\ia\engine-gpu"
    !insertmacro VB_FETCH_URL "Accélération NVIDIA (CUDA)" "${CUDA_URL}" "${CUDA_MB}" "$INSTDIR\ia\engine-gpu"
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
