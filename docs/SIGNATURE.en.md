# Signing

[Français](SIGNATURE.md) · English

## Status

- **Windows code signing** (certificate): **none for now**. The application to [SignPath Foundation](https://signpath.org/) was declined on 2026-10-23. Windows may therefore show a warning at installation. Another solution is still to be chosen (a new application later, a Certum open source certificate, Azure Trusted Signing, the Microsoft Store…).
- **Update signing** (minisign): **yes**. Each release contains the installer, its `.sig` signature and `latest.json`. The application verifies this signature with the public key in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`) and refuses an invalid update. It does not replace code signing: it guarantees that the update comes from the project.
- **Third-party engines** (whisper.cpp, llama.cpp): downloaded as published by their authors, not signed by this project. Antivirus software may block them: see "If your antivirus blocks an engine" in the [README](../README.en.md).

## Verifying a download

Each release's notes give the installer's **SHA-256 checksum**. To check that the downloaded file is identical to the one built by GitHub Actions (PowerShell, in the file's folder):

```powershell
Get-FileHash .\Verbatim_X.Y.Z_x64-setup.exe -Algorithm SHA256
```

Compare the result with the checksum in the notes. This detects a corrupted or tampered file; it is not a code signature (see "Status" above). The engine packs downloaded by the installer are verified automatically by SHA-256 (see `packs.nsh` in [DEVELOPMENT.en.md](DEVELOPMENT.en.md)). Checksums appear only on releases built after they were added (from the next version).

## Update signing key

1. Generation (once): `npx tauri signer generate -w .secrets/verbatim-updater.key`. The `.secrets/` folder and `*.key` files are ignored by git: **never commit the private key**.
2. Public key: in `src-tauri/tauri.conf.json`, `plugins.updater.pubkey`.
3. Private key: in the GitHub secret `TAURI_SIGNING_PRIVATE_KEY` (Settings › Secrets and variables › Actions), plus a backup copy outside the repository. If the key has a password: secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. Losing the private key means republishing the application with a new public key: already installed versions could no longer update by themselves.

## Publishing a release

See "Publishing a release" in [DEVELOPMENT.en.md](DEVELOPMENT.en.md): bump the version, update `CHANGELOG.md`, then `git tag vX.Y.Z && git push origin vX.Y.Z`. `.github/workflows/release.yml` builds, signs the update and publishes the release.
