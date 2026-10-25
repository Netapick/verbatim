# Installation codes and error messages — Verbatim

[Français](CODES.md) · English

Updated for version 0.4.0. The application spells the problem out. When Windows is the one refusing or failing, it adds the system code in parentheses (“os error 5”). The installer has its own codes, visible in its log. This note covers all three. The message texts below are translated; the application shows them in the language of its interface.

## 1. The installer

### Log
The installation of components (engines, GPU acceleration) writes what it does to `redist-install.log`, at the root of the installation folder. Lines starting with `[hook]` come from the installer, the others from the download script. It is the first file to ask for when an installation goes wrong.

### Graphics card detection code
The `detect-gpu.ps1` script returns a number in which each bit stands for a type of card. It appears in the log as `[hook] GPU detection, code: N` (French installs print `détection GPU, code : N`).

| Value | Card detected | Component installed |
|---|---|---|
| 1 | NVIDIA (any series) | Whisper CUDA (685 MB), CUDA report-writing AI (254 MB) and CUDA libraries (373 MB) |
| 2 | AMD Radeon RX 7000 or PRO W7000 (RDNA 3) | ROCm pack (Whisper and report-writing AI) |
| 4 | Any other AMD card (before the RX 7000 series), or Intel | Whisper Vulkan (17 MB) |

Values add up when several cards are present.

| Code | Meaning |
|---|---|
| 0 | No card recognised: everything runs on the processor |
| 1 | NVIDIA |
| 2 | AMD RX 7000 |
| 3 | NVIDIA + AMD RX 7000 |
| 4 | Older AMD, or Intel |
| 5 | NVIDIA + Intel (or older AMD) |
| 6 | AMD RX 7000 + Intel (typically a processor with integrated graphics) |
| 7 | All three |

For value 4, “Basic”, “Remote” and “Virtual” adapters (virtual machines, remote desktop) are ignored. A computer with no recognised GPU is fully usable: transcription and report writing run on the processor.

### Download return code
Each component is downloaded by `dl-pack.ps1`. The log shows `[hook] return code: N`.

| Code | Meaning |
|---|---|
| 0 | Downloaded, verified (SHA-256 checksum) and installed |
| 1 | Failed: the `ERROR:` line before it gives the cause |

Possible causes on the `ERROR:` line

| Message | Cause | What to do |
|---|---|---|
| Download failed (code N) | `curl` failed; see the table below | Check the Internet connection, then run the installer again |
| Invalid checksum: the file is corrupted or was modified | The file received is not the expected one | Run the installer again; if it persists, a proxy or an antivirus is altering the download |
| Invalid archive | The zip contains a path that leaves the installation folder | Do not continue: download the installer again from the official page |

The `N` in “Download failed” is the `curl` exit code:

| N | Meaning |
|---|---|
| 6 | Server name not found (DNS, no connection) |
| 7 | Cannot connect (firewall, proxy) |
| 22 | The server answered with an error (404: the file does not exist at that address) |
| 28 | Timeout |
| 35 | Secure connection (SSL/TLS) failed |
| 56 | Connection dropped during the download |
| 60 | Server certificate rejected (wrong computer clock, proxy inspecting the traffic) |

A failure does not stop the installation: Verbatim works, without the component concerned. Just run the installer again. Components already present are not downloaded again.

### Installer exit code
It follows the NSIS convention: `0` installation succeeded, `1` cancelled by the user, `2` aborted by the script.

### Command-line options
Used by the built-in update; useful for deploying without an interface.

| Option | Effect |
|---|---|
| `/S` | Silent: no window or dialog |
| `/P` | Passive: a progress bar, no questions (the mode used by the application's update) |
| `/UPDATE` | Marks an update: keeps the data and the components already installed |
| `/R` | Restarts the application when finished |
| `/D=<folder>` | Installation folder (must be the last option) |

Components downloaded during installation are in `redist`. Models and engines the application downloads afterwards are in `ia`.

## 2. Application messages

### Installation and downloads
| Message | Cause | What to do |
|---|---|---|
| Whisper engine not installed. / Install the Whisper engine first (Settings). | No transcription engine | Click “Install Whisper” (card under “Meetings”, or Settings › Transcription) |
| No Whisper model installed: install at least the “Base” model | The engine is there but no model | Settings › Transcription |
| A download is already in progress. | One download at a time | Wait for it to finish, or “Stop” |
| Download impossible: … | Network, or server unavailable | Check the connection; the download resumes where it stopped |
| Download cancelled. | You clicked “Stop” | Nothing: the part already received is deleted |
| Cannot write (disk full?) | No space left on the disk | Free some space |
| Access denied to “…”: your antivirus (or Windows Defender) blocks or locks this file | An antivirus is scanning or quarantining the file | See “Antivirus” below |
| Your antivirus deleted or quarantined: … | An unsigned AI engine was removed right after extraction | Restore the files in Windows Security and exclude the `ia` folder |
| … missing from the archive | Incomplete engine archive | Try again; if it persists, the antivirus removed the engine |

### Transcription and speaker separation
| Message | Cause | What to do |
|---|---|---|
| Empty or unreadable audio. / No audio track. | Damaged file or unsupported format | Convert the file (mp3, wav) and import it again |
| Transcription crashed (unreadable audio file?) | The engine stopped during the job | Try again, or import the file again |
| No speech detected. | The speech detector found nothing | Check the microphone level and the file |
| The transcript is empty. | No text produced | Try a larger model |
| Timeout while loading Whisper. | The engine did not start in time (often the antivirus or the graphics driver) | Choose “Processor” in Settings › Transcription, exclude the `ia` folder from the antivirus |
| Live transcription needs whisper-server, missing from this Whisper engine | Old engine without the server | Reinstall the transcription engine |
| Whisper is not loaded yet: wait for loading to finish before starting | Preloading is in progress (live mode) | Wait a few seconds |
| Download the speaker separation models first | Models missing | “Install speaker separation” |
| Speaker separation models unreadable: delete them and download them again | Damaged files | Settings › Transcription, delete then reinstall |
| Speaker separation failed. / … crashed. | Error during the calculation | Run “Identify speakers” again |

### Recording
| Message | Cause | What to do |
|---|---|---|
| No microphone detected. / No audio output detected. | Device not found | Plug the device in, or pick another device in the recording panel |
| Cannot open the audio device … (check Windows Settings › Privacy › Microphone) | Windows denies access | Allow desktop apps to use the microphone |
| Timeout while opening the audio device. | The device does not answer | Unplug it and plug it back in |
| Choose at least one source. | Neither microphone nor PC sound | Tick at least one source |
| A recording is already in progress. | One recording at a time | Finish the first one |
| Recording too short (less than one second). | Nothing to keep | Start again |
| Cannot create the recording file | Disk full or inaccessible folder | Free some space |

After a crash or a power cut, the recording is recovered at the next launch under the title “Recovered recording”.

### Report and translation
| Message | Cause | What to do |
|---|---|---|
| No AI model available: install a model in Settings › Report | No AI chosen | “Report model not installed” card |
| The local AI engine (llama.cpp) is not installed | Engine missing | Download a model: the engine comes with it |
| Timeout while loading the local AI. | Often a model too large for the memory, or a stopped engine | Pick a smaller model or “CPU only” |
| … the AI engine was stopped, often by the antivirus | Runtime (Vulkan, CUDA, ROCm) quarantined | Restore in Windows Security, exclude `ia`, or pick “CPU only” |
| Model answer cut off: it exceeds the maximum allowed length | Most often a very long meeting | Try again; a larger model helps |
| Empty answer from the model. | The AI produced nothing | Try again |
| A report is already being generated. / A translation is already in progress. | One task at a time | Wait |
| The transcription is not finished yet. / The meeting is not ready. | The meeting is still being processed | Wait for it to finish |
| This translation does not exist yet. | Exporting a language that was not translated | Translate first |
| ROCm runtime not found in this folder (expected: amdhip64_7.dll and rocblas.dll) | The folder given is not a ROCm runtime | Pick the folder that contains these two files |

### Update
| Message | Cause | What to do |
|---|---|---|
| A recording / a transcription / a report / a translation / a download is in progress. | The update is refused while a job is running | Wait for it to finish, then try again |
| Possible update errors: offline, invalid signature, interrupted download | Network, or a modified file | “Retry”; an invalid signature means the file was altered: do not insist |

### System errors (“os error N”)
When Windows refuses an operation on a file or a connection, the application's message ends with `(os error N)`, preceded by Windows' own text in the system language. Example: “Access is denied. (os error 5)”. The most common ones:

| Code | Windows name | Meaning | What to do |
|---|---|---|---|
| 2, 3 | File or path not found | A file or folder disappeared (moved, deleted, or removed by the antivirus) | Reinstall the component concerned |
| 5 | Access denied | Insufficient rights, or an antivirus blocks the file | See “Antivirus”; do not install in a protected folder (`Program Files`) |
| 8, 1455 | Not enough memory | Not enough memory or page file to load the model | Close other applications, pick a smaller model, or increase Windows' page file |
| 32, 33 | Sharing or lock violation | Another program (often an antivirus scanning) is using the file | Wait a few seconds and try again; exclude the `ia` folder |
| 80, 183 | File already exists | A file with the same name is present | Try again; delete the offending file |
| 112 | Not enough space on the disk | Disk full | Free some space (models weigh several GB) |
| 123, 206 | Invalid name, or path too long | Installation folder too deep | Pick a shorter folder |
| 225, 226 | File contains a virus (removed) | An antivirus removed the file | See “Antivirus” |
| 1224 | File open in another program | An antivirus is scanning the file right after extraction | Try again; exclude the `ia` folder |
| 10053, 10054, 10061 | Connection aborted, reset, refused | The AI engine stopped or was killed midway | See “Antivirus”; otherwise pick “CPU only” |
| 10060 | Connection timed out | Network or firewall | Try again; check the firewall |

Codes 5, 32, 33 and 1224 trigger the message “Access denied to …: your antivirus … blocks or locks this file”. Codes 10053, 10054 and 10061 trigger the message “the AI engine was stopped, often by the antivirus”.

### Antivirus
The AI engines (whisper.cpp, llama.cpp) are not signed: some antivirus programs delete them. If a message mentions “access denied”, a “stopped” engine or a file “missing from the archive”:

1. Open Windows Security › Virus & threat protection › Protection history.
2. Restore the Verbatim files.
3. Add the `ia` folder (next to `verbatim.exe`) to the exclusions.
4. As a last resort, pick “CPU only” in Settings › Report.
