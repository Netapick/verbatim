# Antivirus false positives: submission template

[Français](FAUX-POSITIFS.md) · English

For maintainers. While the installer is unsigned (see [SIGNATURE.en.md](SIGNATURE.en.md)), every new version is an unknown file to antivirus products (reputation is per file hash). Only submit if an antivirus really blocks the published version.

## Before sending

1. Download the installer **from the GitHub release** (not a local build: its hash would differ).
2. Check its hash: `Get-FileHash .\Verbatim_X.Y.Z_x64-setup.exe -Algorithm SHA256`, and compare it with the one in the release notes.
3. Note the antivirus' exact message and, if any, the **detection name** (`Win32:…`, `Trojan:Win32/…`). A block "for analysis" (Avast CyberCapture) or a SmartScreen "Unknown publisher" screen is a reputation issue, not a detection.
4. If a downloaded engine (`llama-server.exe`, `whisper-server.exe`) is quarantined, submit it separately with the same text.

## Where to submit

| Vendor | Case | Form |
|---|---|---|
| Avast / AVG | Block or detection | Avast "False positive" page (fields: detection name, alert ID, website URL, description) |
| Microsoft | Defender Antivirus detection | [File submission](https://www.microsoft.com/en-us/wdsi/filesubmission): "Software developer" profile, "Incorrectly detected as malware/malicious", detection name required |
| Microsoft | SmartScreen only | Same form, but no detection name: limited effect, code signing remains the real fix |

Replace `X.Y.Z` and the SHA-256 in the text below. Keep the same publisher name everywhere (**Netapick**).

## Text to paste

```
This is a false positive. Verbatim is a free, open-source (MIT) Windows desktop application that transcribes and summarizes meetings entirely on the user's PC (no cloud, no account, no telemetry).

Source code: https://github.com/Netapick/verbatim
Release: https://github.com/Netapick/verbatim/releases/tag/vX.Y.Z
File: Verbatim_X.Y.Z_x64-setup.exe
SHA-256: <hash>
Privacy: https://github.com/Netapick/verbatim/blob/main/docs/PRIVACY.en.md

Why it may look suspicious:
- The installer is new, rarely downloaded and not code-signed yet.
- The app is built with Tauri (Rust + WebView2). At first run it downloads open-source AI engines from their official releases: whisper.cpp and llama.cpp (github.com/ggml-org), and models from Hugging Face. These engines are unsigned executables that the app launches as local child processes.
- It records audio from the microphone and system sound (WASAPI loopback) at the user's request, which is the core feature of a meeting transcriber.

The installer is built by GitHub Actions from the public source. Its auto-update is verified with a Tauri minisign signature.

Please review and remove the detection. Thank you.
```

This version is about 1,200 characters (Microsoft's field limit: 1,900).

### Variant: reputation block (Avast CyberCapture, SmartScreen)

Replace the first sentence with:

```
Avast (CyberCapture) held this file for analysis. I believe this is a reputation issue, not malware: the installer is not code-signed yet.
```

or, for SmartScreen:

```
Microsoft Defender SmartScreen shows "Windows protected your PC - Unknown publisher" when running this installer. I believe this is a reputation issue, not malware: the installer is not code-signed yet.
```

Detection name field when there is none: `CyberCapture - file held for analysis (no detection name)` or `SmartScreen warning - Unknown publisher (no AV detection name)`.
