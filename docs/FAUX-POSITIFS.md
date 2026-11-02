# Faux positifs antivirus : modèle de soumission

Français · [English](FAUX-POSITIFS.en.md)

Document à l'usage des mainteneurs. Tant que l'installateur n'est pas signé (voir [SIGNATURE.md](SIGNATURE.md)), chaque nouvelle version est un fichier inconnu pour les antivirus (réputation par somme du fichier). Ne soumettre que si un antivirus bloque réellement la version publiée.

## Avant d'envoyer

1. Télécharger l'installateur **depuis la release GitHub** (pas une copie construite en local : la somme serait différente).
2. Vérifier sa somme : `Get-FileHash .\Verbatim_X.Y.Z_x64-setup.exe -Algorithm SHA256`, à comparer à celle des notes de release.
3. Relever le message exact de l'antivirus et, s'il existe, le **nom de détection** (`Win32:…`, `Trojan:Win32/…`). Un blocage « pour analyse » (Avast CyberCapture) ou un écran SmartScreen « Éditeur inconnu » est un problème de réputation, pas une détection.
4. Si un moteur téléchargé (`llama-server.exe`, `whisper-server.exe`) est mis en quarantaine, le soumettre à part avec le même texte.

## Où soumettre

| Éditeur | Cas | Formulaire |
|---|---|---|
| Avast / AVG | Blocage ou détection | Page « False positive » d'Avast (champs : nom de détection, ID d'alerte, URL du site, description) |
| Microsoft | Détection Defender Antivirus | [Soumission de fichier](https://www.microsoft.com/en-us/wdsi/filesubmission) : profil « Software developer », « Incorrectly detected as malware/malicious », nom de détection obligatoire |
| Microsoft | Seulement SmartScreen | Même formulaire, mais aucun nom de détection : l'effet est limité, la signature de code reste la vraie solution |

Remplacer `X.Y.Z` et `SHA-256` dans le texte ci-dessous. Garder le même nom d'éditeur partout (**Netapick**). Les formulaires sont en anglais.

## Texte à coller

```
This is a false positive. Verbatim is a free, open-source (MIT) Windows desktop application that transcribes and summarizes meetings entirely on the user's PC (no cloud, no account, no telemetry).

Source code: https://github.com/Netapick/verbatim
Release: https://github.com/Netapick/verbatim/releases/tag/vX.Y.Z
File: Verbatim_X.Y.Z_x64-setup.exe
SHA-256: <somme>
Privacy: https://github.com/Netapick/verbatim/blob/main/docs/PRIVACY.en.md

Why it may look suspicious:
- The installer is new, rarely downloaded and not code-signed yet.
- The app is built with Tauri (Rust + WebView2). At first run it downloads open-source AI engines from their official releases: whisper.cpp and llama.cpp (github.com/ggml-org), and models from Hugging Face. These engines are unsigned executables that the app launches as local child processes.
- It records audio from the microphone and system sound (WASAPI loopback) at the user's request, which is the core feature of a meeting transcriber.

The installer is built by GitHub Actions from the public source. Its auto-update is verified with a Tauri minisign signature.

Please review and remove the detection. Thank you.
```

Cette version fait environ 1 200 caractères (limite du champ Microsoft : 1 900).

### Variante : blocage par réputation (Avast CyberCapture, SmartScreen)

Remplacer la première phrase par :

```
Avast (CyberCapture) held this file for analysis. I believe this is a reputation issue, not malware: the installer is not code-signed yet.
```

ou, pour SmartScreen :

```
Microsoft Defender SmartScreen shows "Windows protected your PC - Unknown publisher" when running this installer. I believe this is a reputation issue, not malware: the installer is not code-signed yet.
```

Champ « nom de détection » sans nom : `CyberCapture - file held for analysis (no detection name)` ou `SmartScreen warning - Unknown publisher (no AV detection name)`.
