# Construit le dossier "redist" livré avec l'installateur de Verbatim :
# moteurs (whisper.cpp, llama.cpp CPU / Vulkan, option ROCm) et modèle de détection de parole.
# Les modèles de transcription et de rédaction ne sont PAS inclus : l'utilisateur les installe depuis les Réglages.
#
#   .\scripts\build-redist.ps1                 # CPU + Vulkan (~70 Mo)
#   .\scripts\build-redist.ps1 -WithRocm       # + ROCm AMD RDNA3 (~1,4 Go de plus)
param(
    [switch]$WithRocm,
    [string]$WhisperRocmFrom = "",  # dossier "bin" de whisper.cpp compilé en ROCm (voir build-whisper-rocm.ps1)
    [string]$RocmRuntimeFrom = ""   # dossier déjà prêt (DLL ROCm) au lieu de télécharger les wheels AMD
)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$out = Join-Path $root "redist"
$tmp = Join-Path $env:TEMP "verbatim-redist"
New-Item -ItemType Directory -Force $out, $tmp | Out-Null

$WHISPER = "https://github.com/ggml-org/whisper.cpp/releases/download/v1.9.2/whisper-bin-x64.zip"
$VAD     = "https://huggingface.co/ggml-org/whisper-vad/resolve/main/ggml-silero-v5.1.2.bin"
$LL      = "https://github.com/ggml-org/llama.cpp/releases/download/b11700"
$AMD     = "https://rocm.nightlies.amd.com/v2/gfx110X-dgpu"
$ROCM_VER = "7.10.0a20251120"

function Get-File($url, $dst) {
    if (Test-Path $dst) { return }
    Write-Host "  téléchargement $(Split-Path $url -Leaf)"
    & curl.exe -sSL --fail -o $dst $url
    if ($LASTEXITCODE -ne 0) { throw "Échec du téléchargement : $url" }
}
# extrait un zip en aplatissant les dossiers (comme le fait l'application)
function Expand-Flat($zip, $dest) {
    New-Item -ItemType Directory -Force $dest | Out-Null
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $z = [IO.Compression.ZipFile]::OpenRead($zip)
    try {
        foreach ($e in $z.Entries) {
            if ($e.Name) { [IO.Compression.ZipFileExtensions]::ExtractToFile($e, (Join-Path $dest $e.Name), $true) }
        }
    } finally { $z.Dispose() }
}
function Check($path, $what) { if (-not (Test-Path $path)) { throw "$what absent : $path" } }

Write-Host "whisper.cpp (CPU)"
Get-File $WHISPER "$tmp\whisper.zip"
Expand-Flat "$tmp\whisper.zip" "$out\engine"
Check "$out\engine\whisper-cli.exe" "whisper-cli.exe"

Write-Host "Détection de parole (VAD)"
New-Item -ItemType Directory -Force "$out\models" | Out-Null
Get-File $VAD "$out\models\ggml-silero-v5.1.2.bin"

Write-Host "llama.cpp (CPU)"
Get-File "$LL/llama-b11700-bin-win-cpu-x64.zip" "$tmp\llama-cpu.zip"
Expand-Flat "$tmp\llama-cpu.zip" "$out\llm"
Check "$out\llm\llama-server.exe" "llama-server.exe (CPU)"

Write-Host "llama.cpp (Vulkan : AMD, NVIDIA, Intel)"
Get-File "$LL/llama-b11700-bin-win-vulkan-x64.zip" "$tmp\llama-vulkan.zip"
Expand-Flat "$tmp\llama-vulkan.zip" "$out\llm-vulkan"
Check "$out\llm-vulkan\llama-server.exe" "llama-server.exe (Vulkan)"

if ($WithRocm) {
    Write-Host "llama.cpp (ROCm)"
    Get-File "$LL/llama-b11700-bin-win-rocm-10.0-x64.zip" "$tmp\llama-rocm.zip"
    Expand-Flat "$tmp\llama-rocm.zip" "$out\llm-rocm"
    Check "$out\llm-rocm\llama-server.exe" "llama-server.exe (ROCm)"

    if ($WhisperRocmFrom) {
        Write-Host "Whisper (ROCm)"
        $we = "$out\engine-rocm"; New-Item -ItemType Directory -Force $we | Out-Null
        foreach ($f in "whisper-cli.exe", "whisper-server.exe", "whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll", "ggml-hip.dll") { Copy-Item "$WhisperRocmFrom\$f" $we -Force }
    } elseif (-not (Test-Path "$out\engine-rocm\whisper-cli.exe")) {
        Write-Warning "Pas de Whisper ROCm : compilez-le avec scripts/build-whisper-rocm.ps1 puis relancez avec -WhisperRocmFrom."
    }

    Write-Host "Runtime ROCm (RDNA3 / gfx110X)"
    $rt = "$out\rocm-runtime"
    New-Item -ItemType Directory -Force $rt | Out-Null
    if ($RocmRuntimeFrom) {
        # dossier site-packages d'un venv AMD, ou dossier déjà aplati
        $sp = $RocmRuntimeFrom
        if (Test-Path "$sp\_rocm_sdk_core") { $core = "$sp\_rocm_sdk_core"; $lib = (Get-ChildItem $sp -Directory -Filter "_rocm_sdk_libraries*" | Select-Object -First 1).FullName }
        else { throw "Attendu : un dossier site-packages contenant _rocm_sdk_core" }
    } else {
        $wc = "rocm_sdk_core-$ROCM_VER-py3-none-win_amd64.whl"
        $wl = "rocm_sdk_libraries_gfx110x_dgpu-$ROCM_VER-py3-none-win_amd64.whl"
        Get-File "$AMD/$wc" "$tmp\core.zip"
        Get-File "$AMD/$wl" "$tmp\libs.zip"
        Remove-Item "$tmp\wheels" -Recurse -Force -ErrorAction SilentlyContinue
        Expand-Archive "$tmp\core.zip" "$tmp\wheels" -Force
        Expand-Archive "$tmp\libs.zip" "$tmp\wheels" -Force
        $core = "$tmp\wheels\_rocm_sdk_core"
        $lib = (Get-ChildItem "$tmp\wheels" -Directory -Filter "_rocm_sdk_libraries*" | Select-Object -First 1).FullName
    }
    # sous-ensemble minimal vérifié : HIP + comgr (core) ; hipBLAS, rocBLAS, hipBLASLt, rocSOLVER et leurs bibliothèques de noyaux
    Copy-Item "$core\bin\*.dll" $rt -Force
    foreach ($f in "hipblas.dll", "rocblas.dll", "libhipblaslt.dll", "rocsolver.dll") { Copy-Item "$lib\bin\$f" $rt -Force }
    Copy-Item "$lib\bin\rocblas" "$rt\rocblas" -Recurse -Force
    Copy-Item "$lib\bin\hipblaslt" "$rt\hipblaslt" -Recurse -Force
    Check "$rt\amdhip64_7.dll" "amdhip64_7.dll"
}

$size = (Get-ChildItem $out -Recurse -File | Measure-Object Length -Sum).Sum / 1MB
Write-Host ("`nredist prêt : {0} ({1:N0} Mo)" -f $out, $size) -ForegroundColor Green
