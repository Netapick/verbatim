# Prépare les packs téléchargés par l'installateur (installateur léger) à partir du dossier "redist".
#
#   .\scripts\make-packs.ps1 -BaseUrl "https://exemple.com/verbatim/0.1.0"
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
    [string[]]$Rebuild = @("CORE", "ROCM")
)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$redist = Join-Path $root "redist"
$out = Join-Path $root "dist\packs"
New-Item -ItemType Directory -Force $out | Out-Null
$ver = (Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json).version

$packs = [ordered]@{
    CORE = @{ File = "verbatim-redist-core-0.1.0.zip"; Dirs = @("engine", "llm", "llm-vulkan", "models") }
    ROCM = @{ File = "verbatim-redist-rocm-0.3.0.zip"; Dirs = @("engine-rocm", "llm-rocm", "rocm-runtime") }  # 0.3.0 : + whisper-server ROCm (direct)
}

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$nsh = @("; Généré par scripts/make-packs.ps1 : ne pas modifier à la main.", "!define PACKS_BASE `"$($BaseUrl.TrimEnd('/'))`"")
foreach ($k in $packs.Keys) {
    $pk = $packs[$k]
    $zip = Join-Path $out $pk.File
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
        } finally { $za.Dispose(); $fs.Dispose() }
    }
    if (-not (Test-Path $zip)) { throw "Pack absent : $zip (lancez sans -SkipZip)" }
    $mb = [int][math]::Ceiling((Get-Item $zip).Length / 1MB)
    $sha = (Get-FileHash -Algorithm SHA256 $zip).Hash
    Write-Host ("  {0} Mo  SHA256 {1}" -f $mb, $sha)
    $nsh += "!define PACK_${k}_FILE `"$($pk.File)`""
    $nsh += "!define PACK_${k}_SHA `"$sha`""
    $nsh += "!define PACK_${k}_MB `"$mb`""
}
$dst = Join-Path $root "src-tauri\installer\packs.nsh"
[IO.File]::WriteAllText($dst, ($nsh -join "`r`n") + "`r`n", (New-Object Text.UTF8Encoding($true)))
Write-Host "packs.nsh écrit : $dst"
