# Installation codes and error messages — Verbatim

[Français](CODES.md) · English

Updated for version 0.4.1. The application spells the problem out; when Windows is the one refusing or failing, it adds the system code in parentheses (“os error 5”). The installer has its own codes, visible in its log. The message texts below are translated; the application shows them in the language of its interface.

## Where to start

| What you see | Go to |
|---|---|
| “Access denied”, a “stopped” engine, a file “missing from the archive”, `os error 5`, `32`, `1224`, `10054` | [Antivirus](#antivirus-the-most-common-cause) |
| The installation failed or a component is missing | [The installer](#1-the-installer) |
| A message in the application | [Application messages](#2-application-messages) |
| An `(os error N)` at the end of a message | [System errors](#system-errors-os-error-n) |

**Contents** · [Antivirus](#antivirus-the-most-common-cause) · [1. The installer](#1-the-installer) · [2. Application messages](#2-application-messages) · [System errors](#system-errors-os-error-n)

---

## Antivirus, the most common cause

The AI engines (whisper.cpp, llama.cpp) are not signed: some antivirus programs delete or block them. If a message mentions “access denied”, a “stopped” engine or a file “missing from the archive”:

1. Open **Windows Security › Virus & threat protection › Protection history**.
2. Restore the Verbatim files.
3. Add the `ia` folder (next to `verbatim.exe`) to the **exclusions**.
4. As a last resort, pick **“CPU only”** in Settings › Report.

---

## 1. The installer

> [!TIP]
> First file to ask for when an installation goes wrong: **`redist-install.log`**, at the root of the installation folder. Lines starting with `[hook]` come from the installer, the others from the download script.

### Graphics card detection code

The `detect-gpu.ps1` script returns a number in which each bit stands for a type of card. The log shows `[hook] GPU detection, code: N` (French installs print `détection GPU, code : N`).

| Bit | Card detected | Component installed |
|:-:|---|---|
| **1** | NVIDIA (any series) | Whisper CUDA (685 MB), CUDA report-writing AI (254 MB) and CUDA libraries (373 MB) |
| **2** | AMD Radeon RX 7000 or PRO W7000 (RDNA 3) | ROCm pack (Whisper and report-writing AI) |
| **4** | Any other AMD card (before the RX 7000 series), or Intel | Whisper Vulkan (17 MB) |

Values add up when several cards are present:

| `0` | `1` | `2` | `3` | `4` | `5` | `6` | `7` |
|---|---|---|---|---|---|---|---|
| No card recognised (everything on the processor) | NVIDIA | AMD RX 7000 | NVIDIA + AMD RX 7000 | Older AMD, or Intel | NVIDIA + Intel (or older AMD) | AMD RX 7000 + Intel (typically a processor with integrated graphics) | All three |

For value 4, “Basic”, “Remote” and “Virtual” adapters (virtual machines, remote desktop) are ignored. A computer with no recognised GPU is fully usable: transcription and report writing run on the processor.

### Downloading a component

Each component is downloaded by `dl-pack.ps1`. The log shows `[hook] return code: N`.

- **`0`**: downloaded, verified (SHA-256 checksum) and installed.
- **`1`**: failed. The `ERROR:` line before it gives the cause.

| Message on the `ERROR:` line | Cause and solution |
|---|---|
| Download failed (code N) | `curl` failed (codes in the table below).<br>**→** Check the Internet connection, then run the installer again. |
| Invalid checksum: the file is corrupted or was modified | The file received is not the expected one.<br>**→** Run the installer again; if it persists, a proxy or an antivirus is altering the download. |
| Invalid archive | The zip contains a path that leaves the installation folder.<br>**→** Do not continue: download the installer again from the official page. |

<details>
<summary><b><code>curl</code> codes</b> (the <code>N</code> in “Download failed”)</summary>

| N | Meaning |
|:-:|---|
| 6 | Server name not found (DNS, no connection) |
| 7 | Cannot connect (firewall, proxy) |
| 22 | The server answered with an error (404: the file does not exist at that address) |
| 28 | Timeout |
| 35 | Secure connection (SSL/TLS) failed |
| 56 | Connection dropped during the download |
| 60 | Server certificate rejected (wrong computer clock, proxy inspecting the traffic) |

</details>

> [!NOTE]
> A failure does not stop the installation: Verbatim works, without the component concerned. Just run the installer again; components already present are not downloaded again.

<details>
<summary><b>Exit code and command-line options</b></summary>

**Installer exit code** (NSIS convention): `0` installation succeeded · `1` cancelled by the user · `2` aborted by the script.

**Command-line options**, used by the built-in update and useful for deploying without an interface:

| Option | Effect |
|---|---|
| `/S` | Silent: no window or dialog |
| `/P` | Passive: a progress bar, no questions (the mode used by the application's update) |
| `/UPDATE` | Marks an update: keeps the data and the components already installed |
| `/R` | Restarts the application when finished |
| `/D=<folder>` | Installation folder (must be the last option) |

</details>

**Where are the files?** Components downloaded during installation are in `redist`. Models and engines the application downloads afterwards are in `ia`.

---

## 2. Application messages

In each table, the second column gives the cause and then, after **→**, what to do.

<details>
<summary><b>Installation and downloads</b> · 9 messages</summary>

| Message | Cause → what to do |
|---|---|
| Whisper engine not installed. / Install the Whisper engine first (Settings). | No transcription engine.<br>**→** Click “Install Whisper” (card under “Meetings”, or Settings › Transcription). |
| No Whisper model installed: install at least the “Base” model | The engine is there but no model.<br>**→** Settings › Transcription. |
| A download is already in progress. | One download at a time.<br>**→** Wait for it to finish, or “Stop”. |
| Download impossible: … | Network, or server unavailable.<br>**→** Check the connection; the download resumes where it stopped. |
| Download cancelled. | You clicked “Stop”.<br>**→** Nothing: the part already received is deleted. |
| Cannot write (disk full?) | No space left on the disk.<br>**→** Free some space. |
| Access denied to “…”: your antivirus (or Windows Defender) blocks or locks this file | An antivirus is scanning or quarantining the file.<br>**→** See [Antivirus](#antivirus-the-most-common-cause). |
| Your antivirus deleted or quarantined: … | An unsigned AI engine was removed right after extraction.<br>**→** Restore the files in Windows Security and exclude the `ia` folder. |
| … missing from the archive | Incomplete engine archive.<br>**→** Try again; if it persists, the antivirus removed the engine. |

</details>

<details>
<summary><b>Transcription and speaker separation</b> · 10 messages</summary>

| Message | Cause → what to do |
|---|---|
| Empty or unreadable audio. / No audio track. | Damaged file or unsupported format.<br>**→** Convert the file (mp3, wav) and import it again. |
| Transcription crashed (unreadable audio file?) | The engine stopped during the job.<br>**→** Try again, or import the file again. |
| No speech detected. | The speech detector found nothing.<br>**→** Check the microphone level and the file. |
| The transcript is empty. | No text produced.<br>**→** Try a larger model. |
| Timeout while loading Whisper. | The engine did not start in time (often the antivirus or the graphics driver).<br>**→** Choose “Processor” in Settings › Transcription, exclude the `ia` folder from the antivirus. |
| Live transcription needs whisper-server, missing from this Whisper engine | Old engine without the server.<br>**→** Reinstall the transcription engine. |
| Whisper is not loaded yet: wait for loading to finish before starting | Preloading is in progress (live mode).<br>**→** Wait a few seconds. |
| Download the speaker separation models first | Models missing.<br>**→** “Install speaker separation”. |
| Speaker separation models unreadable: delete them and download them again | Damaged files.<br>**→** Settings › Transcription, delete then reinstall. |
| Speaker separation failed. / … crashed. | Error during the calculation.<br>**→** Run “Identify speakers” again. |

</details>

<details>
<summary><b>Recording</b> · 7 messages</summary>

| Message | Cause → what to do |
|---|---|
| No microphone detected. / No audio output detected. | Device not found.<br>**→** Plug the device in, or pick another device in the recording panel. |
| Cannot open the audio device … (check Windows Settings › Privacy › Microphone) | Windows denies access.<br>**→** Allow desktop apps to use the microphone. |
| Timeout while opening the audio device. | The device does not answer.<br>**→** Unplug it and plug it back in. |
| Choose at least one source. | Neither microphone nor PC sound.<br>**→** Tick at least one source. |
| A recording is already in progress. | One recording at a time.<br>**→** Finish the first one. |
| Recording too short (less than one second). | Nothing to keep.<br>**→** Start again. |
| Cannot create the recording file | Disk full or inaccessible folder.<br>**→** Free some space. |

> [!NOTE]
> After a crash or a power cut, the recording is recovered at the next launch under the title “Recovered recording”.

</details>

<details>
<summary><b>Report and translation</b> · 10 messages</summary>

| Message | Cause → what to do |
|---|---|
| No AI model available: install a model in Settings › Report | No AI chosen.<br>**→** “Report model not installed” card. |
| The local AI engine (llama.cpp) is not installed | Engine missing.<br>**→** Download a model: the engine comes with it. |
| Timeout while loading the local AI. | Often a model too large for the memory, or a stopped engine.<br>**→** Pick a smaller model or “CPU only”. |
| … the AI engine was stopped, often by the antivirus | Runtime (Vulkan, CUDA, ROCm) quarantined.<br>**→** Restore in Windows Security, exclude `ia`, or pick “CPU only”. |
| Model answer cut off: it exceeds the maximum allowed length | Most often a very long meeting.<br>**→** Try again; a larger model helps. |
| Empty answer from the model. | The AI produced nothing.<br>**→** Try again. |
| A report is already being generated. / A translation is already in progress. | One task at a time.<br>**→** Wait. |
| The transcription is not finished yet. / The meeting is not ready. | The meeting is still being processed.<br>**→** Wait for it to finish. |
| This translation does not exist yet. | Exporting a language that was not translated.<br>**→** Translate first. |
| ROCm runtime not found in this folder (expected: amdhip64_7.dll and rocblas.dll) | The folder given is not a ROCm runtime.<br>**→** Pick the folder that contains these two files. |

</details>

<details>
<summary><b>Update</b> · 2 messages</summary>

| Message | Cause → what to do |
|---|---|
| A recording / a transcription / a report / a translation / a download is in progress. | The update is refused while a job is running.<br>**→** Wait for it to finish, then try again. |
| Possible update errors: offline, invalid signature, interrupted download | Network, or a modified file.<br>**→** “Retry”; an invalid signature means the file was altered: do not insist. |

</details>

---

## System errors, os error N

When Windows refuses an operation on a file or a connection, the application's message ends with `(os error N)`, preceded by Windows' own text in the system language. Example: “Access is denied. (os error 5)”.

| Code | Windows name | Cause → what to do |
|:-:|---|---|
| **2**, **3** | File or path not found | A file or folder disappeared (moved, deleted, or removed by the antivirus).<br>**→** Reinstall the component concerned. |
| **5** | Access denied | Insufficient rights, or an antivirus blocks the file.<br>**→** See [Antivirus](#antivirus-the-most-common-cause); do not install in a protected folder (`Program Files`). |
| **8**, **1455** | Not enough memory | Not enough memory or page file to load the model.<br>**→** Close other applications, pick a smaller model, or increase Windows' page file. |
| **32**, **33** | Sharing or lock violation | Another program (often an antivirus scanning) is using the file.<br>**→** Wait a few seconds and try again; exclude the `ia` folder. |
| **80**, **183** | File already exists | A file with the same name is present.<br>**→** Try again; delete the offending file. |
| **112** | Not enough space on the disk | Disk full.<br>**→** Free some space (models weigh several GB). |
| **123**, **206** | Invalid name, or path too long | Installation folder too deep.<br>**→** Pick a shorter folder. |
| **225**, **226** | File contains a virus (removed) | An antivirus removed the file.<br>**→** See [Antivirus](#antivirus-the-most-common-cause). |
| **1224** | File open in another program | An antivirus is scanning the file right after extraction.<br>**→** Try again; exclude the `ia` folder. |
| **10053**, **10054**, **10061** | Connection aborted, reset, refused | The AI engine stopped or was killed midway.<br>**→** See [Antivirus](#antivirus-the-most-common-cause); otherwise pick “CPU only”. |
| **10060** | Connection timed out | Network or firewall.<br>**→** Try again; check the firewall. |

Codes 5, 32, 33 and 1224 trigger the message “Access denied to …: your antivirus … blocks or locks this file”. Codes 10053, 10054 and 10061 trigger the message “the AI engine was stopped, often by the antivirus”.
