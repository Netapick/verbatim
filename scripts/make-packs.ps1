# Prépare les packs téléchargés par l'installateur (installateur léger) à partir du dossier "redist".
#
#   .\scripts\make-packs.ps1 -BaseUrl "https://github.com/Netapick/verbatim/releases/download"
#   .\scripts\make-packs.ps1 -BaseUrl "..." -SkipZip       # ne régénère que packs.nsh (nouvelle URL)
#   .\scripts\make-packs.ps1 -BaseUrl "..." -Rebuild ROCM  # ne refait que le pack ROCm (les autres zips existants sont réutilisés)
#
# Chaque pack a sa propre version dans son nom : un pack modifié reçoit un NOUVEAU nom (jamais le même fichier avec un autre
# contenu), car les anciens installateurs vérifient la somme SHA-256 de l'ancien fichier.
#
# Produit dist\packs\*.zip (à mettre en ligne à l'adresse -BaseUrl) et src-tauri\installer\packs.nsh
# (tailles, sommes SHA-256 et URL lues par l'installateur). Relancez ensuite : npx tauri build
param(
    [Parameter(Mandatory)][string]$BaseUrl,
    [switch]$SkipZip,
    [string[]]$Rebuild = @("CORE", "ROCM", "ROCMFIX")
)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$redist = Join-Path $root "redist"
$out = Join-Path $root "dist\packs"
New-Item -ItemType Directory -Force $out | Out-Null
$ver = (Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json).version

$packs = [ordered]@{
    CORE = @{ Tag = "v0.1.0"; File = "verbatim-redist-core-0.1.0.zip"; Dirs = @("engine", "llm", "llm-vulkan", "models") }
    ROCM = @{ Tag = "v0.3.0"; File = "verbatim-redist-rocm-0.3.0.zip"; Dirs = @("engine-rocm", "llm-rocm", "rocm-runtime") }  # 0.3.0 : + whisper-server ROCm (direct)
    # correctif pour les mises à jour : seul whisper-server.exe, ajouté à un dossier engine-rocm déjà installé (~0,5 Mo)
    ROCMFIX = @{ Tag = "v0.3.0"; File = "verbatim-redist-rocm-server-0.3.0.zip"; Files = @("engine-rocm\whisper-server.exe") }
    # Whisper Vulkan (AMD d'avant RX 7000, Intel) : ~17 Mo ; facultatif tant que redist\engine-vulkan n'existe pas (voir build-whisper-vulkan.ps1)
    VULKAN = @{ Tag = "v0.4.0"; File = "verbatim-redist-vulkan-0.4.0.zip"; Dirs = @("engine-vulkan"); Optional = $true }
}

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$nsh = @("; Généré par scripts/make-packs.ps1 : ne pas modifier à la main.")
foreach ($k in $packs.Keys) {
    $pk = $packs[$k]
    $zip = Join-Path $out $pk.File
    # pack facultatif : ignoré (et absent de packs.nsh : l'installateur ne le télécharge pas) tant que ni son dossier ni son zip n'existent
    if ($pk.Optional -and -not (Test-Path $zip) -and -not (Test-Path (Join-Path $redist $pk.Dirs[0]))) { Write-Host "Pack $k ignoré (redist\$($pk.Dirs[0]) absent)"; continue }
    if ($pk.Optional -and -not $SkipZip -and $Rebuild -notcontains $k -and (Test-Path (Join-Path $redist $pk.Dirs[0])) -and -not (Test-Path $zip)) { $Rebuild += $k }
    if (-not $SkipZip -and ($Rebuild -contains $k)) {
        foreach ($d in $pk.Dirs) { if (-not (Test-Path (Join-Path $redist $d))) { throw "Dossier manquant : redist\$d" } }
        Write-Host "Pack $k -> $($pk.File)"
        Remove-Item $zip -Force -ErrorAction SilentlyContinue
        $fs = [IO.File]::Create($zip)
        $za = New-Object IO.Compression.ZipArchive($fs, [IO.Compression.ZipArchiveMode]::Create)
        try {
            foreach ($d in $pk.Dirs) {
                $base = Join-Path $redist $d
                Get-ChildItem $base -Recurse -File | ForEach-Object {
                    $rel = "redist/$d/" + $_.FullName.Substring($base.Length + 1).Replace('\', '/')
                    [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($za, $_.FullName, $rel, [IO.Compression.CompressionLevel]::Optimal) | Out-Null
                }
            }
            foreach ($f in $pk.Files) {
                $src = Join-Path $redist $f
                if (-not (Test-Path $src)) { throw "Fichier manquant : redist\$f" }
                [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($za, $src, "redist/" + $f.Replace('\', '/'), [IO.Compression.CompressionLevel]::Optimal) | Out-Null
            }
        } finally { $za.Dispose(); $fs.Dispose() }
    }
    if (-not (Test-Path $zip)) { throw "Pack absent : $zip (lancez sans -SkipZip)" }
    $mb = [int][math]::Ceiling((Get-Item $zip).Length / 1MB)
    $sha = (Get-FileHash -Algorithm SHA256 $zip).Hash
    Write-Host ("  {0} Mo  SHA256 {1}" -f $mb, $sha)
    # chaque pack est téléchargé depuis la release (tag) où il a été publié
    $nsh += "!define PACK_${k}_BASE `"$($BaseUrl.TrimEnd('/'))/$($pk.Tag)`""
    $nsh += "!define PACK_${k}_FILE `"$($pk.File)`""
    $nsh += "!define PACK_${k}_SHA `"$sha`""
    $nsh += "!define PACK_${k}_MB `"$mb`""
}
$dst = Join-Path $root "src-tauri\installer\packs.nsh"
[IO.File]::WriteAllText($dst, ($nsh -join "`r`n") + "`r`n", (New-Object Text.UTF8Encoding($true)))
Write-Host "packs.nsh écrit : $dst"
