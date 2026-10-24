# Technical documentation

[Français](DEVELOPMENT.md) · English

Verbatim is a **Tauri 2 + Rust** desktop application (HTML/JS interface without a framework in `ui/`), for Windows x64.

## Code layout

| Folder / file | Role |
|---|---|
| `ui/` | Interface: `app.js` (views), `i18n.js` (translation), `style.css`, `index.html` |
| `src-tauri/src/main.rs` | Data (meetings, preparations, glossary, settings), Tauri commands |
| `src-tauri/src/engine.rs` | Transcription: whisper.cpp, audio decoding (symphonia), resumable downloads, GPU detection |
| `src-tauri/src/llm.rs` | Writing: llama.cpp (local server), Ollama, remote API, simplified summary |
| `src-tauri/src/recorder.rs` | Microphone and PC sound recording (cpal, WASAPI loopback); sample copy for live mode |
| `src-tauri/src/live.rs` | Live transcription: persistent `whisper-server`, sliding window, `live-*` events |
| `src-tauri/src/translate.rs` | Batched transcript translation (TranslateGemma or the report model) |
| `src-tauri/src/models.rs` | Detection and copy of existing models |
| `src-tauri/src/update.rs` | Automatic update |
| `src-tauri/installer/` | NSIS hooks and PowerShell scripts of the installer |
| `scripts/` | Building the engines and packs, logo |

## Report language, translation and live mode

- **Report language**: `Settings.report_lang` (`auto` or a code), overridden per meeting through `generate_report(lang)`; `llm::rules(lang)` adds the language instruction, `report.lang` remembers the choice, `section_labels` localizes the export headings.
- **Translation**: `translate_meeting(id, lang)` splits segments into batches of 8, with a single `llama-server` for the whole meeting (`llm::with_backend`). TranslateGemma is called with a raw prompt on `/completion` (its chat template rejects the system role); other models go through `chat_text`. The result is stored in `Meeting.translations[lang]` with the original timestamps; compute is set by `translate_device` (`llm::translate_cfg`). Stop: `translate_stop` (flag checked between batches). Models live in `ia/translation-models`.
- **Live mode**: `recorder::Sink` publishes samples into one `TapBuf` per source; `live.rs` mixes them, sends the not-yet-committed audio tail to `whisper-server` (`/inference`) every ~0.6 s, commits sentences that ended 1.2 s before the end of the audio (`live-segment` event), leaves the last one provisional (`live-partial`) and translates each committed sentence (`live-translation`). Server: ROCm (`engine-rocm/whisper-server.exe`) if a Radeon card and the runtime are present, otherwise CPU; models up to small on CPU. The server is restarted if it stops (up to 3 times, log `verbatim-whisper-server.log`); live speech detection is done in Rust (`earshot` crate, 16 ms frames) rather than by the server's VAD. The cpal capture callback only copies samples to a processing thread (resampling, disk, live). On stop, `recorder::finish` shuts live mode down, then the normal transcription of the file starts.

Data lives in a `store.json` file plus the copied audio files, in the application's data folder.

## Recording

Microphone, PC sound (WASAPI loopback: video calls, videos) or both, mixed to 16 kHz mono, with device choice (remembered), pause/resume, timer and live levels. Audio is written to disk as you go: after a crash or power loss, the recording is rebuilt on the next launch (about 2 s lost at most).

## Transcription

- whisper.cpp v1.9.2 (official CPU binary); audio is decoded in Rust (symphonia: mp3, m4a/aac, ogg, flac, wav), converted to 16 kHz mono and transcribed offline. The glossary is passed to Whisper as the initial prompt. Speech detection (Silero VAD) limits hallucinations on silences. Greedy decoding (`-bs 1`, about 30% faster than `-bs 5`): the shorter segments are merged per sentence (6 s at most) and a sentence repeated more than twice in a row is cut back to two. Decoding and resampling run on two threads. Beyond one hour (GPU only), the WAV is split into 3 at silences (`split_points`) and transcribed by 3 parallel instances (about 43% faster on 1 h), timestamps shifted when stitching; whisper-cli's `-ot`/`-d` do not suit the VAD (they count time after silence removal). Measured separately: running the report in parallel (`llama-server -np 3`) only gains 21% for 15.6 GB of VRAM out of 16, so it stays sequential.
- Models: tiny, base, small, medium-q5_0, large-v3-turbo-q5_0, large-v3-turbo-q8_0, large-v3-turbo, large-v3-q5_0, downloaded from Hugging Face (`ggerganov/whisper.cpp`).
- Resumable downloads: `.part` file kept, HTTP Range resume, progress kept across page changes; a stop button (`download_cancel`).
- Computation: "Automatic" mode (GPU when available, fallback to the CPU) or "CPU only". NVIDIA: official whisper.cpp CUDA build (b5454, ≈ 685 MB, `ia/engine-gpu`). AMD: whisper.cpp has no official binary, so `scripts/build-whisper-rocm.ps1` builds it for HIP/ROCm on RDNA3 (≈ 180 MB, `redist/engine-rocm`).

## Report writing

Order of priority: local AI (llama.cpp + Qwen, loaded only while writing), then Ollama (started on demand), then an OpenAI-compatible API, then a simplified summary computed from the transcript (real sentences, no invented text). Long meetings are split into parts, summarized, then merged. The glossary and the linked preparation are provided to the model.

### Writing computation engines

| Engine | Folder | Notes |
|---|---|---|
| CPU | `ia/llm` | Works everywhere |
| Vulkan | `ia/llm-vulkan` | AMD, NVIDIA, Intel; ≈ 32 MB |
| ROCm | `ia/llm-rocm` | AMD RDNA3; runtime shipped in `redist/rocm-runtime` or chosen by hand |
| CUDA | `ia/llm-cuda` | NVIDIA; two llama.cpp archives (engine + CUDA DLLs) |

Automatic mode: CUDA, then ROCm, then Vulkan, then CPU; if the GPU engine is cut off while writing, one retry is made on the CPU. A model larger than the available video memory is flagged in Settings (read from the Windows registry, `engine::gpu_vram`).

Speed and resource usage measurements (RAM, VRAM, processor): see the "Performance and resource usage" section of the [README](../README.en.md). The charts are generated by `scripts/make-benchmark-charts.py` (output in `docs/benchmarks/`, with the raw data `results.json`).

## Engines and models: locations

Engines and models are stored in `ia/` next to `verbatim.exe` (otherwise in the application's data if that folder is not writable). Engine search order: `ia/` (installed by the user), then `redist/` (shipped). Settings › Models detects existing models (an older Verbatim folder, Ollama's models, a `.gguf` / `ggml-*.bin` you choose) and **copies** them into `ia`.

## Installer and distribution

The installer is a lightweight NSIS (≈ 3 MB, per user, French and English). Engines are downloaded during installation, with no questions:

1. Base pack (≈ 61 MB: whisper.cpp, VAD, llama.cpp CPU and Vulkan).
2. Graphics card detection (`src-tauri/installer/detect-gpu.ps1`), then the ROCm pack (Radeon RX 7000 / PRO W7000, ≈ 414 MB; on an existing install where only `whisper-server.exe` is missing, just the small ≈ 0.5 MB patch is downloaded) or the CUDA engines (NVIDIA: Whisper b5454 and llama.cpp b11700 + CUDA DLLs).
3. The chosen language is written to `lang.txt`, picked up by the application at first launch.

The packs are zips published on the repository's releases: base pack on `v0.1.0`, ROCm packs (full and the `whisper-server` patch) on `v0.3.0`. A modified pack always gets a **new name** (old installers check the SHA-256 of the old file). `packs.nsh` (each pack's URL, size, SHA-256) is generated by `scripts/make-packs.ps1`. Installation log: `redist-install.log`.

```
./scripts/build-redist.ps1              # whisper.cpp + VAD + llama.cpp CPU and Vulkan
./scripts/build-redist.ps1 -WithRocm    # + AMD RDNA3 ROCm (minimal runtime)
python scripts/make-hiproot.py <site-packages with rocm_sdk_devel> <hiproot>   # rebuilds the ROCm development SDK (once)
./scripts/build-whisper-rocm.ps1 -HipRoot <hiproot>   # builds whisper-cli and whisper-server for ROCm
./scripts/make-packs.ps1 -BaseUrl <url> # pack zips + packs.nsh (-Rebuild ROCM for a single pack)
npx tauri build                         # NSIS installer
```

## Automatic updates

At startup, the app queries `releases/latest` of the GitHub repository, downloads the installer (`*_x64-setup.exe`), verifies its SHA-256 (the API's `digest` field), runs it silently in the same folder, then restarts (`src-tauri/src/update.rs`). Never during a recording, a transcription or report writing. Can be turned off in Settings › Connections.

## Antivirus

The third-party engines are unsigned and may be deleted by an antivirus. After each extraction, `engine::verify_kept` waits for the scan then checks that the files are still there, and `engine::fs_err` turns access-denied errors into an explicit message. See the "If your antivirus blocks an engine" section of the [README](../README.en.md).

## Interface translation

The interface texts are written in French in the code. `ui/i18n.js` holds a French → English dictionary and patterns for messages with variable values, applied at display time (text nodes and attributes). To translate a new text, add its pair to `EN` (key = exact French text) or a pattern to `P`. The language choice is in Settings › Connections; by default, the installer's language (`lang.txt`), then the system's.

## Building

```
npm install
npx tauri dev                    # run in development
npx tauri build --no-bundle      # exe only: src-tauri/target/release/verbatim.exe
npx tauri build                  # installer (requires NSIS)
```

## Publishing a version

1. Bump the version in `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`, add an entry to `CHANGELOG.md`, then commit.
2. Push a `vX.Y.Z` tag: the `.github/workflows/release.yml` workflow builds the installer, has it signed by SignPath and publishes the release (see [SIGNATURE.en.md](SIGNATURE.en.md)). The release must be the most recent one for the automatic update to see it.

## Visual identity

Logo: `python scripts/make-logo.py` (app tile drawn as a raster: gradients, glow, reflection; simplified versions at 16/24/32 px); exports `src-tauri/icons/*` (ico + png) and `ui/logo.png`, `ui/favicon.ico`, `ui/favicon.png`.
