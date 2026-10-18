# Code signing (SignPath)

[Français](SIGNATURE.md) · English

The installer is signed by [SignPath Foundation](https://signpath.org/) (free for open source projects; the repository is under the MIT license). Signing happens on GitHub Actions: `.github/workflows/release.yml` builds the installer, sends it to SignPath, retrieves the signed version and publishes the release.

## What is signed, and what is not
- **Signed**: the installer `Verbatim_X.Y.Z_x64-setup.exe`.
- **Not signed**: the downloaded third-party engines (whisper.cpp, llama.cpp). SignPath only signs the project's own files. Antivirus software may therefore still block them: see "If your antivirus blocks an engine" in the presentation.

## Setup (one time, by the repository owner)
1. Apply at <https://signpath.org/apply> for the `Netapick/verbatim` repository (MIT license, public repository). Approval is manual.
2. In SignPath, create the project with slug `verbatim`, with:
   - an **artifact configuration**: a zip file containing `*_x64-setup.exe` (PE type, to be signed);
   - a **signing policy** with slug `release-signing`, linked to the GitHub repository (origin verification).
3. Install the SignPath GitHub app on the repository, then add in GitHub (Settings › Secrets and variables › Actions):
   - secret `SIGNPATH_API_TOKEN`: API token of a SignPath CI user;
   - variable `SIGNPATH_ORGANIZATION_ID`: the SignPath organization identifier.

## Publishing a signed version
1. Bump the version in `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`, then commit.
2. `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. The workflow publishes the release with the signed installer. The application's automatic update reads this release (`releases/latest`) and verifies its SHA-256.

Note: the repository no longer publishes locally built executables; earlier versions (0.1.0 to 0.2.1) remain unsigned.
