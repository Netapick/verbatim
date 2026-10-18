# Verbatim

[Français](README.md) · English

**Transcribe your meetings and get written reports, without anything leaving your computer.**

[![Release](https://img.shields.io/github/v/release/Netapick/verbatim)](https://github.com/Netapick/verbatim/releases/latest) [![MIT license](https://img.shields.io/github/license/Netapick/verbatim)](LICENSE) [![Downloads](https://img.shields.io/github/downloads/Netapick/verbatim/total)](https://github.com/Netapick/verbatim/releases) ![Windows x64](https://img.shields.io/badge/Windows-x64-0078d4) ![NVIDIA CUDA](https://img.shields.io/badge/NVIDIA-CUDA-76B900?logo=nvidia&logoColor=white) ![AMD ROCm](https://img.shields.io/badge/AMD-ROCm%20%2B%20Vulkan-ED1C24?logo=amd&logoColor=white)

[**Download Verbatim**](https://github.com/Netapick/verbatim/releases/latest) · [Illustrated presentation](docs/PRESENTATION.en.md) · [Privacy](docs/PRIVACY.en.md)

![Meeting report](docs/screenshots/03-compte-rendu.png)

Verbatim is a Windows desktop application. It records or imports your meetings, transcribes them with **Whisper**, then writes the report with a **Qwen** AI that runs on your own machine. Audio, transcripts and models stay with you: no account, no subscription, no telemetry. (The screenshots show the French interface; the app is also available in English.)

## Features

**Record or import**
- Record the microphone, the PC sound (video calls, videos) or both, with pause, live levels and recovery after a crash.
- Import audio (mp3, m4a/aac, ogg, flac, wav…) or an existing transcript (txt, srt, vtt, md).

**Transcribe**
- Local Whisper (whisper.cpp), with speech detection to limit hallucinations on silences.
- French, English, Spanish, German, Italian, or automatic detection; a glossary of acronyms is passed to the model.

**Write**
- Summary, topics, decisions, actions and next steps, by a local AI (Qwen 3.5 4B/9B, Qwen 3.8 27B, or your own GGUF models). Ollama and an OpenAI-compatible API are optional.
- Without a writing model, a simplified summary is built from real sentences of the transcript, inventing nothing.

**Prepare and look back**
- Meeting preparations (objective, agenda, questions), customizable meeting types.
- Audio player synchronized with the transcript (click a passage to play from there), export to text, subtitles, JSON, Markdown and audio.
- Light and dark theme, French and English interface, automatic updates.

## Supported hardware

| Graphics card | Transcription (Whisper) | Writing (Qwen) |
|---|---|---|
| **NVIDIA** | CUDA | CUDA, otherwise Vulkan |
| **AMD Radeon RX 7000 / PRO W7000** (RDNA 3) | ROCm | ROCm, otherwise Vulkan |
| Other AMD, Intel | Processor | Vulkan |
| No graphics card | Processor | Processor |

whisper.cpp has no official Windows binary for AMD cards: Verbatim ships a ROCm build compiled for Radeon RX 7000 cards, making GPU-accelerated transcription possible on AMD as well as NVIDIA. "Automatic" mode picks the fastest available engine, with fallback to the processor.

Writing speed (RX 7800 XT, Qwen 3.5 4B Q4, prompt / generation in tokens per second): processor 94 / 11, Vulkan 2566 / 127, ROCm 3673 / 117.

## Installation

1. Download `Verbatim_X.Y.Z_x64-setup.exe` from the [latest release](https://github.com/Netapick/verbatim/releases/latest). The installer is lightweight (about 3 MB) and offers French or English.
2. It downloads the engines by itself: the base pack (≈ 61 MB), then the acceleration that matches your card (ROCm ≈ 430 MB, or CUDA ≈ 1.3 GB). It asks no questions and needs no administrator rights.
3. Launch Verbatim, then download a transcription model and, if you wish, a writing model from **Settings**. No model is shipped with the application.

Engines and models are stored in an `ia` folder next to the application. If you already have models (Ollama, `.gguf` or `ggml-*.bin` files), **Settings › Models** detects them and **copies** them into `ia`, leaving the originals untouched.

**Updates:** at startup, Verbatim looks for the latest version on GitHub, verifies its SHA-256, installs it and restarts. Never during a recording, a transcription or report writing. Can be turned off in **Settings › Connections**.

### If your antivirus blocks an engine

The third-party engines (whisper.cpp, llama.cpp) are unsigned: some antivirus products, including Windows Defender, may delete or block them ("access denied" or "connection closed" errors). Add Verbatim's installation folder (the one containing `ia`) to the exclusions: *Windows Security › Virus & threat protection › Manage settings › Exclusions › Add an exclusion › Folder*. Then restore the files from the protection history, or restart the download. Running the application as administrator does not help.

## Privacy

Audio, transcripts and reports are processed **only on your computer**. The only outgoing connections: downloading engines and models (GitHub, Hugging Face) at your request, the update check against the GitHub API, and, only if you configure one yourself, a remote API for report writing. Full text: [docs/PRIVACY.en.md](docs/PRIVACY.en.md).

## Limitations

- Windows x64 only.
- No speaker separation for now: a single "Intervenant" (speaker).
- Reports are written in the meeting's language (the instructions given to the AI are in French).

## Documentation

- [Illustrated presentation](docs/PRESENTATION.en.md): a tour of the application, with screenshots.
- [Technical documentation](docs/DEVELOPMENT.en.md): architecture, building, distribution, updates.
- [Code signing](docs/SIGNATURE.en.md) · [Privacy](docs/PRIVACY.en.md)

## License

Code under the [MIT](LICENSE) license. Third-party components (whisper.cpp, llama.cpp, ROCm) remain under their respective licenses, detailed in each release's `THIRD-PARTY-LICENSES.md`.

### Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

- Only the installer built from this repository by GitHub Actions (`.github/workflows/release.yml`) is signed; each signing request is manually approved.
- Roles: [Netapick](https://github.com/Netapick) is author, reviewer and approver (multi-factor authentication enabled).
- Third-party engines (whisper.cpp, llama.cpp) are downloaded as published by their authors and are not signed by this project.
