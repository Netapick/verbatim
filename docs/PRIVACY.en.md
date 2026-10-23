# Privacy Policy — Verbatim

[Français](PRIVACY.md) · English

Last updated: October 18, 2026.

## Your data stays on your computer
Verbatim records, transcribes and summarizes your meetings **locally**. Audio, transcripts, reports, meeting preparations and the glossary are stored on your computer, in the application's folder. The project collects no personal data, runs no server, and includes no telemetry, tracking or advertising.

## The application's only network connections
1. **Downloading engines and models**, only when you ask for it (or during installation): GitHub (whisper.cpp and llama.cpp engines, installer packs) and Hugging Face (Whisper, Qwen and TranslateGemma models). These services see a standard download request, as with a web browser.
2. **Update check**: at startup, the application queries the GitHub API (`releases/latest` of the `Netapick/verbatim` repository) and, if a newer version exists, downloads the installer and verifies its SHA-256 checksum. Nothing about you is sent beyond what an HTTPS request contains (IP address, application version in the `User-Agent` header). You can turn these updates off in **Settings › Connections**.
3. **Remote writing API (optional)**: if, and only if, you yourself configure an API address and key in Settings, the transcript text is sent to that service to write the report or, if no local model is installed, to translate it. Its handling is then governed by that provider's privacy policy. Without configuration, nothing is sent.

## Third-party components
Verbatim relies on whisper.cpp and llama.cpp (run locally) and on models downloaded from Hugging Face. They run on your machine; their respective licenses apply (see `THIRD-PARTY-LICENSES.md` in the releases).

## Deleting your data
Uninstalling the application removes its program files; the `ia` folder (models) can be kept or deleted at uninstall. Your meetings can be deleted from within the application (Trash) or by deleting its data folder.

## Contact
For any question, open an [issue](https://github.com/Netapick/verbatim/issues) on the repository.
