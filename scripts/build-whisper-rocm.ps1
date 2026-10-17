# Compile whisper.cpp avec ROCm/HIP (aucun binaire officiel n'existe). Prérequis :
#   - Visual Studio Build Tools 2022 (C++), Python avec : pip install cmake ninja rocm-sdk-core rocm-sdk-devel rocm-sdk-libraries-custom
#   - SDK ROCm de développement dans un dossier $HipRoot (copie de _rocm_sdk_core + en-têtes, .lib et cmake extraits de rocm_sdk_devel\_devel.tar)
# Résultat : <Work>\whisper.cpp-1.9.2\build-hip\bin  (à passer à build-redist.ps1 -WhisperRocmFrom)
param(
    [Parameter(Mandatory)][string]$HipRoot,
    [string]$Work = (Join-Path $env:TEMP "verbatim-whisper-rocm"),
    [string]$Targets = "gfx1100;gfx1101;gfx1102"   # RDNA3
)
$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force $Work | Out-Null
$src = "$Work\whisper.cpp-1.9.2"
if (-not (Test-Path $src)) {
    & curl.exe -sSL --fail -o "$Work\src.zip" https://github.com/ggml-org/whisper.cpp/archive/refs/tags/v1.9.2.zip
    Expand-Archive "$Work\src.zip" $Work -Force
}
$vc = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
$py = (& python -c "import sys,os;print(os.path.dirname(sys.executable))").Trim()
$bat = "$Work\b.bat"
@"
@echo off
call "$vc" >nul
set HIP_PATH=$HipRoot
set PATH=$HipRoot\bin;$HipRoot\lib\llvm\bin;$py\Scripts;%PATH%
cd /d "$src"
cmake -S . -B build-hip -G Ninja -DCMAKE_BUILD_TYPE=Release -DGGML_HIP=ON "-DAMDGPU_TARGETS=$Targets" -DCMAKE_C_COMPILER="$HipRoot\lib\llvm\bin\clang.exe" -DCMAKE_CXX_COMPILER="$HipRoot\lib\llvm\bin\clang++.exe" -DWHISPER_BUILD_TESTS=OFF -DBUILD_SHARED_LIBS=ON -DCMAKE_PREFIX_PATH="$HipRoot" -DHIP_PLATFORM=amd
if errorlevel 1 exit /b 1
cmake --build build-hip -j 8
"@ | Set-Content $bat -Encoding ascii
cmd /c $bat
if ($LASTEXITCODE -ne 0) { throw "Compilation échouée" }
Write-Host "OK : $src\build-hip\bin"
