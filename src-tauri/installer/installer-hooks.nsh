; Installateur léger : les moteurs et accélérations (dossier « redist ») sont téléchargés pendant l'installation.
; Les tailles, sommes de contrôle et l'URL viennent de packs.nsh (généré par scripts/make-packs.ps1).
; Langues : l'installateur propose le français et l'anglais ; les textes ci-dessous suivent $LANGUAGE (1033 = anglais).
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

; Texte détaillé : français ou anglais selon la langue choisie.
!macro VB_SAY FR EN
  ${If} $LANGUAGE = 1033
    DetailPrint "${EN}"
  ${Else}
    DetailPrint "${FR}"
  ${EndIf}
!macroend

!macro VB_FETCH LABEL_FR LABEL_EN BASE FILE SHA MB
  !insertmacro VB_SAY "${LABEL_FR} (environ ${MB} Mo)…" "${LABEL_EN} (about ${MB} MB)…"
  !insertmacro VB_LOG "[hook] ${LABEL_FR}"
  InitPluginsDir
  File "/oname=$PLUGINSDIR\dl-pack.ps1" "${VB_DIR}\dl-pack.ps1"
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "$PLUGINSDIR\dl-pack.ps1" -Url "${BASE}/${FILE}" -Sha "${SHA}" -Dest "$INSTDIR" -SizeMb ${MB} -Lang $R7'
  Pop $0
  !insertmacro VB_LOG "[hook] code retour : $0"
  ${If} $0 != "0"
    !insertmacro VB_SAY "Échec : ${LABEL_FR}." "Failed: ${LABEL_EN}."
    ${IfNot} ${Silent}
      ${If} $LANGUAGE = 1033
        MessageBox MB_OK|MB_ICONEXCLAMATION "Downloading “${LABEL_EN}” failed.$\r$\n$\r$\nCheck your Internet connection. Verbatim will work, but without these components: you can run the installer again to add them."
      ${Else}
        MessageBox MB_OK|MB_ICONEXCLAMATION "Le téléchargement de « ${LABEL_FR} » a échoué.$\r$\n$\r$\nVérifiez votre connexion Internet. Verbatim fonctionnera, mais sans ces composants : vous pourrez relancer l'installateur pour les ajouter."
      ${EndIf}
    ${EndIf}
  ${EndIf}
!macroend

; Moteur Whisper CUDA officiel (NVIDIA), extrait à plat dans ia\engine-gpu : même source que le bouton des Réglages.
!define CUDA_URL "https://github.com/ggml-org/whisper.cpp/releases/download/b5454/whisper-bin-win-cuda-12.4.0-x64.zip"
!define CUDA_MB "685"
; Moteur llama.cpp CUDA (rédaction) + DLL runtime : deux archives extraites dans ia\llm-cuda.
!define LLM_CUDA_URL "https://github.com/ggml-org/llama.cpp/releases/download/b11700/llama-b11700-bin-win-cuda-12.4-x64.zip"
!define LLM_CUDA_MB "254"
!define LLM_CUDART_URL "https://github.com/ggml-org/llama.cpp/releases/download/b11700/cudart-llama-bin-win-cuda-12.4-x64.zip"
!define LLM_CUDART_MB "373"

!macro VB_FETCH_URL LABEL_FR LABEL_EN URL MB DEST
  !insertmacro VB_SAY "${LABEL_FR} (environ ${MB} Mo)…" "${LABEL_EN} (about ${MB} MB)…"
  !insertmacro VB_LOG "[hook] ${LABEL_FR}"
  InitPluginsDir
  File "/oname=$PLUGINSDIR\dl-pack.ps1" "${VB_DIR}\dl-pack.ps1"
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "$PLUGINSDIR\dl-pack.ps1" -Url "${URL}" -Dest "${DEST}" -SizeMb ${MB} -Log "$INSTDIR\redist-install.log" -Flat -Lang $R7'
  Pop $0
  !insertmacro VB_LOG "[hook] code retour : $0"
  ${If} $0 != "0"
    !insertmacro VB_SAY "Échec : ${LABEL_FR}." "Failed: ${LABEL_EN}."
    ${IfNot} ${Silent}
      ${If} $LANGUAGE = 1033
        MessageBox MB_OK|MB_ICONEXCLAMATION "Downloading “${LABEL_EN}” failed.$\r$\n$\r$\nVerbatim will run on the processor: you can try again from the Settings."
      ${Else}
        MessageBox MB_OK|MB_ICONEXCLAMATION "Le téléchargement de « ${LABEL_FR} » a échoué.$\r$\n$\r$\nVerbatim fonctionnera sur le processeur : vous pourrez réessayer depuis les Réglages."
      ${EndIf}
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro VB_LOG "[hook] POSTINSTALL"
  ; Langue choisie : transmise aux scripts PowerShell et mémorisée pour l'application (lang.txt, écrit une seule fois).
  ${If} $LANGUAGE = 1033
    StrCpy $R7 "en"
  ${Else}
    StrCpy $R7 "fr"
  ${EndIf}
  ${IfNot} ${FileExists} "$INSTDIR\lang.txt"
    FileOpen $R6 "$INSTDIR\lang.txt" w
    FileWrite $R6 "$R7"
    FileClose $R6
  ${EndIf}
  ; Information (pas une question) : les moteurs d'IA ne sont pas signés, l'antivirus peut les supprimer.
  !insertmacro VB_SAY "Antivirus : ajoutez une exclusion pour le dossier $INSTDIR\ia" "Antivirus: add an exclusion for the folder $INSTDIR\ia"
  ${IfNot} ${Silent}
    ${If} $LANGUAGE = 1033
      MessageBox MB_OK|MB_ICONEXCLAMATION "Antivirus: add a security exclusion for the “ia” folder.$\r$\n$\r$\nThe AI engines (whisper.cpp, llama.cpp) are not signed: your antivirus may delete or block them while they are installed.$\r$\n$\r$\nFolder to exclude:$\r$\n$INSTDIR\ia$\r$\n$\r$\nWindows Defender: Windows Security > Virus & threat protection > Manage settings > Exclusions > Add an exclusion > Folder.$\r$\n$\r$\nDo it now if possible, then click OK: the engines installation then continues."
    ${Else}
      MessageBox MB_OK|MB_ICONEXCLAMATION "Antivirus : ajoutez une exclusion de sécurité pour le dossier « ia ».$\r$\n$\r$\nLes moteurs d'IA (whisper.cpp, llama.cpp) ne sont pas signés : votre antivirus peut les supprimer ou les bloquer pendant leur installation.$\r$\n$\r$\nDossier à exclure :$\r$\n$INSTDIR\ia$\r$\n$\r$\nWindows Defender : Sécurité Windows > Protection contre les virus et menaces > Gérer les paramètres > Exclusions > Ajouter une exclusion > Dossier.$\r$\n$\r$\nFaites-le maintenant si possible, puis cliquez sur OK : l'installation des moteurs continue ensuite."
    ${EndIf}
  ${EndIf}
  ${IfNot} ${FileExists} "$INSTDIR\redist\engine\*.*"
    !insertmacro VB_FETCH "Moteurs de transcription et d'IA" "Transcription and AI engines" "${PACK_CORE_BASE}" "${PACK_CORE_FILE}" "${PACK_CORE_SHA}" "${PACK_CORE_MB}"
  ${EndIf}
  ; Accélération GPU : détectée puis installée directement, sans question.
  InitPluginsDir
  File "/oname=$PLUGINSDIR\detect-gpu.ps1" "${VB_DIR}\detect-gpu.ps1"
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "$PLUGINSDIR\detect-gpu.ps1" -Lang $R7'
  Pop $2
  !insertmacro VB_LOG "[hook] détection GPU, code : $2"
  IntOp $1 $2 & 2
  ${If} $1 != 0
    ${IfNot} ${FileExists} "$INSTDIR\redist\engine-rocm\*.*"
      !insertmacro VB_FETCH "Accélération AMD (ROCm)" "AMD acceleration (ROCm)" "${PACK_ROCM_BASE}" "${PACK_ROCM_FILE}" "${PACK_ROCM_SHA}" "${PACK_ROCM_MB}"
    ${ElseIfNot} ${FileExists} "$INSTDIR\redist\engine-rocm\whisper-server.exe"
      ; mise à jour d'une installation plus ancienne : il ne manque que le serveur de la transcription en direct
      !insertmacro VB_FETCH "Transcription en direct (ROCm)" "Live transcription (ROCm)" "${PACK_ROCMFIX_BASE}" "${PACK_ROCMFIX_FILE}" "${PACK_ROCMFIX_SHA}" "${PACK_ROCMFIX_MB}"
    ${EndIf}
  ${EndIf}
  IntOp $1 $2 & 1
  ${If} $1 != 0
  ${AndIfNot} ${FileExists} "$INSTDIR\ia\engine-gpu\whisper-cli.exe"
    CreateDirectory "$INSTDIR\ia\engine-gpu"
    !insertmacro VB_FETCH_URL "Accélération NVIDIA (CUDA)" "NVIDIA acceleration (CUDA)" "${CUDA_URL}" "${CUDA_MB}" "$INSTDIR\ia\engine-gpu"
  ${EndIf}
  IntOp $1 $2 & 1
  ${If} $1 != 0
  ${AndIfNot} ${FileExists} "$INSTDIR\ia\llm-cuda\llama-server.exe"
    CreateDirectory "$INSTDIR\ia\llm-cuda"
    !insertmacro VB_FETCH_URL "Rédaction NVIDIA (CUDA)" "NVIDIA report writing (CUDA)" "${LLM_CUDA_URL}" "${LLM_CUDA_MB}" "$INSTDIR\ia\llm-cuda"
    !insertmacro VB_FETCH_URL "Bibliothèques CUDA" "CUDA libraries" "${LLM_CUDART_URL}" "${LLM_CUDART_MB}" "$INSTDIR\ia\llm-cuda"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  RMDir /r "$INSTDIR\redist"
  Delete "$INSTDIR\lang.txt"
  ${IfNot} ${Silent}
    ${If} ${FileExists} "$INSTDIR\ia\*.*"
      ${If} $LANGUAGE = 1033
        MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 "Also delete the downloaded models (the “ia” folder, several GB)?" IDNO vb_keep_models
      ${Else}
        MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 "Supprimer aussi les modèles téléchargés (dossier « ia », plusieurs Go) ?" IDNO vb_keep_models
      ${EndIf}
      RMDir /r "$INSTDIR\ia"
      vb_keep_models:
    ${EndIf}
  ${EndIf}
!macroend
