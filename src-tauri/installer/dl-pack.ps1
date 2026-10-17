# Télécharge un pack de composants (zip) et l'extrait dans le dossier d'installation.
# Appelé par l'installateur (installer-hooks.nsh) ; la sortie s'affiche dans le journal de l'installation.
param(
    [Parameter(Mandatory)][string]$Url,
    [Parameter(Mandatory)][string]$Dest,
    [string]$Sha = "",
    [int]$SizeMb = 0,
    [string]$Log = "",
    [switch]$Flat
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$LogPath = if ($Log) { $Log } else { Join-Path $Dest "redist-install.log" }
function Say($m) { Write-Output $m; try { Add-Content -Path $LogPath -Value ("{0:HH:mm:ss} {1}" -f (Get-Date), $m) -Encoding UTF8 } catch {} }
$zip = Join-Path $env:TEMP ("verbatim-" + [guid]::NewGuid().ToString("N") + ".zip")
try {
    Say "Téléchargement : $Url"
    $p = Start-Process -FilePath "$env:SystemRoot\System32\curl.exe" -ArgumentList @("-sSL", "--fail", "--retry", "3", "-o", "`"$zip`"", "`"$Url`"") -PassThru -WindowStyle Hidden
    $last = Get-Date
    while (-not $p.HasExited) {
        Start-Sleep -Milliseconds 500
        if (((Get-Date) - $last).TotalSeconds -ge 5 -and (Test-Path $zip)) {
            $mb = [int]((Get-Item $zip).Length / 1MB)
            Say ("  {0} Mo{1}" -f $mb, $(if ($SizeMb) { " / " + $SizeMb + " Mo" } else { "" }))
            $last = Get-Date
        }
    }
    if ($p.ExitCode -ne 0) { throw "Échec du téléchargement (code $($p.ExitCode))." }
    if ($Sha) {
        Say "Vérification de l'intégrité…"
        $fs = [IO.File]::OpenRead($zip)
        try { $h = ([BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($fs))).Replace("-", "") } finally { $fs.Dispose() }
        if ($h -ne $Sha.ToUpper()) { throw "Somme de contrôle invalide : le fichier est corrompu ou a été modifié." }
    }
    Say "Installation des composants…"
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $root = [IO.Path]::GetFullPath($Dest).TrimEnd('\') + '\'
    $arc = [IO.Compression.ZipFile]::OpenRead($zip)
    try {
        foreach ($e in $arc.Entries) {
            if (-not $e.Name) { continue }
            $rel = if ($Flat) { $e.Name } else { $e.FullName -replace '/', '\' }
            $target = [IO.Path]::GetFullPath((Join-Path $root $rel))
            if (-not $target.StartsWith($root, [StringComparison]::OrdinalIgnoreCase)) { throw "Archive invalide." }
            [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target)) | Out-Null
            [IO.Compression.ZipFileExtensions]::ExtractToFile($e, $target, $true)
        }
    } finally { $arc.Dispose() }
    Say "Terminé."
    exit 0
} catch {
    Say ("ERREUR : " + $_.Exception.Message)
    exit 1
} finally {
    Remove-Item $zip -Force -ErrorAction SilentlyContinue
}
