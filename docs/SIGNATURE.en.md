# Signing

[Français](SIGNATURE.md) · English

## Status

- **Windows code signing** (certificate): **none for now**. The application to [SignPath Foundation](https://signpath.org/) was declined on 2026-10-23. Windows may therefore show a warning at installation. Another solution is still to be chosen (a new application later, a Certum open source certificate, Azure Trusted Signing, the Microsoft Store…).
- **Update signing** (minisign): **yes**. Each release contains the installer, its `.sig` signature and `latest.json`. The application verifies this signature with the public key in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`) and refuses an invalid update. It does not replace code signing: it guarantees that the update comes from the project.
- **Third-party engines** (whisper.cpp, llama.cpp): downloaded as published by their authors, not signed by this project. Antivirus software may block them: see "If your antivirus blocks an engine" in the [README](../README.en.md).

## Update signing key

1. Generation (once): `npx tauri signer generate -w .secrets/verbatim-updater.key`. The `.secrets/` folder and `*.key` files are ignored by git: **never commit the private key**.
2. Public key: in `src-tauri/tauri.conf.json`, `plugins.updater.pubkey`.
3. Private key: in the GitHub secret `TAURI_SIGNING_PRIVATE_KEY` (Settings › Secrets and variables › Actions), plus a backup copy outside the repository. If the key has a password: secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. Losing the private key means republishing the application with a new public key: already installed versions could no longer update by themselves.

## Publishing a release

See "Publishing a release" in [DEVELOPMENT.en.md](DEVELOPMENT.en.md): bump the version, update `CHANGELOG.md`, then `git tag vX.Y.Z && git push origin vX.Y.Z`. `.github/workflows/release.yml` builds, signs the update and publishes the release.
