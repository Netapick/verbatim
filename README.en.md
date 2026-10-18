# Verbatim

[Français](README.md) · English

> Illustrated presentation: [docs/PRESENTATION.en.md](docs/PRESENTATION.en.md) · [Download](https://github.com/Netapick/verbatim/releases/latest)

Desktop application (Tauri 2 + Rust) to organize your meetings: audio import or
transcript import, report, preparations (briefs), glossary, trash.
Original visual identity, name, icon and interface. The interface is in French.

Data: a `store.json` file + the copied audio files, in the app's data folder.
Optional LLM (Settings): OpenAI-compatible API for reports and briefs.

## Recording from the PC
"● Record" button: microphone, PC sound (WASAPI loopback: video calls, videos…) or both, mixed to 16 kHz mono.
Choice of the input device and of the output to capture (remembered), pause/resume, timer and live levels; when you stop, the meeting is created and transcribed automatically.

Audio is written to disk as you go (nothing is kept in memory). After a crash or power loss,
the recording is rebuilt on the next launch (about 2 s lost at most) and added to the meetings.

## Report
Generated from the transcript, in order of priority: local AI (llama.cpp + Qwen 3.5 4B or 9B, or Qwen 3.8 27B (Q4 or lighter), downloaded in Settings; loaded only while writing), then Ollama (if installed: started on demand, model of your choice in Settings), then an OpenAI-compatible API, then a simplified summary computed from the transcript (real sentences, no invented text). Long meetings are split into parts, summarized, then merged. The glossary and the linked preparation are provided to the model.

### Writing AI computation (CPU / GPU)
Settings › Report: CPU, Vulkan GPU (AMD/NVIDIA/Intel, 32 MB engine, no installation) or ROCm GPU (AMD). Each engine is downloaded into `ia/llm`, `ia/llm-vulkan`, `ia/llm-rocm`.
ROCm reuses the runtime of an AMD ComfyUI (the venv's `rocm_sdk` wheels), detected automatically on the drives or chosen by hand. Automatic fallback ROCm → Vulkan → CPU.
Measurement (RX 7800 XT, Qwen 3.5 4B Q4): CPU 94 / 11 tok/s (read / write), Vulkan 2566 / 127, ROCm 3673 / 117.

## Listening
Audio player in each meeting (speed 0.75× to 2×); clicking a passage of the transcript starts playback at that moment, and the current passage is highlighted.

## Transcription
Local Whisper via whisper.cpp v1.9.2 (official CPU binary): on first use, the app downloads
the engine (~8 MB) and the chosen model (tiny 75 MB, base 142 MB, small 466 MB) into its data folder.
Audio is decoded in Rust (symphonia: mp3, m4a/aac, ogg, flac, wav), converted to 16 kHz mono,
then transcribed offline. The glossary is passed to Whisper as the initial prompt.
Models of your choice in Settings (tiny → large v3). Downloads are resumable (`.part` file kept, HTTP Range resume), their progress survives page changes, and models stay in the app's data folder. Speech detection (Silero VAD) is enabled to limit hallucinations on silences.
Computation: CPU by default; CPU / GPU button in Settings (GPU = official CUDA build of whisper.cpp, NVIDIA cards only, ~685 MB to download into `ia/engine-gpu`).
Limits: no speaker separation (a single "Intervenant"), no AMD/Intel GPU for Whisper, Windows x64.

## Model location
Engines (whisper.cpp, llama.cpp) and models are stored in the `ia` folder next to `verbatim.exe` (otherwise in the app's data if that folder is not writable). No model is provided: you download them in Settings.
If models already exist (an older Verbatim folder, models downloaded by Ollama, or any other folder/`.gguf` / `ggml-*.bin` file you choose), Settings › "Existing models" detects them and **copies** them into `ia`. A copied GGUF is used directly with Verbatim's engine, without Ollama.

## Distributing the application
The `redist/` folder (ready-to-use engines) is bundled in the installer and used in place: no engine download at first launch, and no more dependency on ComfyUI or an installed SDK. Models (transcription, writing) still have to be installed from Settings.

    ./scripts/build-redist.ps1              # whisper.cpp + VAD + llama.cpp CPU and Vulkan (~170 MB)
    ./scripts/build-redist.ps1 -WithRocm    # + AMD RDNA3 ROCm: llama.cpp ROCm and minimal runtime (~1.5 GB more)
    ./scripts/build-whisper-rocm.ps1 -HipRoot <sdk>   # (once) builds whisper.cpp for ROCm, then build-redist.ps1 -WithRocm -WhisperRocmFrom <...\build-hip\bin>
    npx tauri build                          # NSIS installer (per user, writable `ia` folder)

Whisper on AMD GPU (ROCm) has no official binary: `build-whisper-rocm.ps1` builds it (≈180 MB, RDNA3) and places it in `redist/engine-rocm`, with the ROCm runtime. It is chosen automatically when "GPU" is checked and a Radeon card is detected.

The installer shows no question: it installs the base pack, then detects the GPU (`src-tauri/installer/detect-gpu.ps1`) and adds the ROCm pack (Radeon RX 7000 / PRO W7000) or the whisper.cpp CUDA engine (NVIDIA, in `ia/engine-gpu`).

Engine search order: `ia/` (installed by the user) then `redist/` (shipped). For a lightweight installer, build `redist` without `-WithRocm`. The shipped ROCm runtime covers RDNA3 (gfx110X); other AMD cards use Vulkan.
Third-party redistributables (whisper.cpp, llama.cpp, ROCm) have their own licenses: see the LICENSE files in `redist/`.

## Antivirus
The downloaded engines (whisper.cpp, llama.cpp: `whisper-cli.exe`, `llama-server.exe`, Vulkan/CUDA/ROCm DLLs) are unsigned; Defender and other antivirus products may delete them or quarantine them (error 5 "access denied" when downloading, error 10054 "connection closed" while writing a report). After extraction, the app waits for the scan, then checks that the files are still there (`engine::verify_kept`) and shows the steps to follow; if the GPU writing engine is cut off, it retries once on the CPU. User solution: add the installation folder (containing `ia`) to the antivirus exclusions (Windows Security › Virus & threat protection › Exclusions), restore the files from the protection history, then restart the download. Running the app as administrator does not help.

## License and signing
Code under the MIT license (`LICENSE`). The installer is built and signed by SignPath from GitHub Actions (`.github/workflows/release.yml`): see `docs/SIGNATURE.en.md`.

## Code signing policy
Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

- Only the installer built from this repository by GitHub Actions (`.github/workflows/release.yml`) is signed; each signing request is manually approved.
- Roles: [Netapick](https://github.com/Netapick) is author, reviewer and approver (multi-factor authentication enabled).
- Third-party engines (whisper.cpp, llama.cpp) are downloaded as published by their authors and are not signed by this project.

## Privacy
Verbatim processes audio, transcripts and reports **only on the computer**: nothing is sent online, unless you configure a remote API yourself for writing. The application's only outgoing connections: downloading engines and models (GitHub, Hugging Face) at your request, and an update check against the GitHub API at startup (can be turned off in Settings › Connections). No personal data or telemetry is collected. Full text: [docs/PRIVACY.en.md](docs/PRIVACY.en.md).

## Updates
At startup, the app queries `releases/latest` of the GitHub repository, downloads the installer (`*_x64-setup.exe`), verifies its SHA-256 (the API's `digest` field), runs it silently in the same folder, then restarts (`src-tauri/src/update.rs`). Never during a recording, a transcription or report writing. Settings › Connections lets you turn off automatic installation. To publish a version: bump the number in `package.json`, `tauri.conf.json`, `Cargo.toml`, `npx tauri build`, then `gh release create vX.Y.Z <exe>` (the release must be the most recent one).

## Visual identity
Logo: `python scripts/make-logo.py` (app tile drawn as a raster: gradients, glow, reflection; simplified versions at 16/24/32 px); exports `src-tauri/icons/*` (ico + png) and `ui/logo.png`, `ui/favicon.ico`, `ui/favicon.png`.

## Development
    npm install
    npx tauri dev                    # run
    npx tauri build --no-bundle      # exe: src-tauri/target/release/verbatim.exe
    npx tauri build                  # installers (requires WiX/NSIS)
