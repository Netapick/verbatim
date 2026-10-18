# Détecte les cartes graphiques pour l'installateur. Code de sortie : bit 1 = NVIDIA, bit 2 = AMD Radeon RX 7000 / PRO W7000 (RDNA 3).
$ErrorActionPreference = "SilentlyContinue"
# NSIS lit la sortie avec la page de codes ANSI de Windows (sinon « é » s'affiche « , » : page OEM).
try { [Console]::OutputEncoding = [Text.Encoding]::GetEncoding([Globalization.CultureInfo]::CurrentCulture.TextInfo.ANSICodePage) } catch {}
$code = 0
foreach ($g in (Get-CimInstance Win32_VideoController)) {
    $n = [string]$g.Name
    if ($n -match 'NVIDIA') { $code = $code -bor 1 }
    if ($n -match '(AMD|Radeon)' -and $n -match '(RX\s*7\d{3}|PRO\s*W7\d{3})') { $code = $code -bor 2 }
}
Write-Output ("GPU détectés : " + ((Get-CimInstance Win32_VideoController | ForEach-Object { $_.Name }) -join " ; ") + " (code $code)")
exit $code
