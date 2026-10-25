# Compile whisper.cpp avec Vulkan (aucun binaire officiel n'existe pour Windows) : cartes AMD d'avant la série RX 7000, Intel, NVIDIA sans CUDA.
# Prérequis :
#   - Visual Studio Build Tools 2022 (C++), Python avec : pip install cmake ninja
#   - SDK Vulkan de LunarG (https://vulkan.lunarg.com/sdk/home#windows : glslc, en-têtes et vulkan-1.lib), variable VULKAN_SDK ou -VulkanSdk
#     L'exécutable obtenu n'embarque pas Vulkan : c'est le pilote graphique de l'utilisateur qui fournit vulkan-1.dll.
# Résultat : <Work>\whisper.cpp-1.9.2\build-vulkan\bin  (whisper-cli.exe, whisper-server.exe et leurs DLL ;
#   à passer à build-redist.ps1 -WhisperVulkanFrom)
param(
    [string]$VulkanSdk = $env:VULKAN_SDK,
    # dossier court : le sous-projet vulkan-shaders-gen imbrique ses essais CMake très profondément, et rc.exe échoue (RC2136) au-delà de ~230 caractères
    [string]$Work = (Join-Path $env:SystemDrive "vkw")
)
$ErrorActionPreference = "Stop"
if (-not $VulkanSdk -or -not (Test-Path "$VulkanSdk\Bin\glslc.exe")) { throw "SDK Vulkan introuvable (glslc.exe attendu dans <SDK>\Bin) : installez-le puis relancez avec -VulkanSdk <dossier>." }
New-Item -ItemType Directory -Force $Work | Out-Null
$src = "$Work\whisper.cpp-1.9.2"
if (-not (Test-Path $src)) {
    & curl.exe -sSL --fail -o "$Work\src.zip" https://github.com/ggml-org/whisper.cpp/archive/refs/tags/v1.9.2.zip
    if ($LASTEXITCODE -ne 0) { throw "Téléchargement des sources impossible" }
    Expand-Archive "$Work\src.zip" $Work -Force
}
$vc = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
$py = (& python -c "import sys,os;print(os.path.dirname(sys.executable))").Trim()
$bat = "$Work\b.bat"
# GGML_NATIVE=OFF : le processeur de la machine de build ne doit pas être imposé aux autres (AVX2 + FMA sont courants depuis 2013)
@"
@echo off
call "$vc" >nul
set VULKAN_SDK=$VulkanSdk
set PATH=$VulkanSdk\Bin;$py\Scripts;%PATH%
cd /d "$src"
cmake -S . -B build-vulkan -G Ninja -DCMAKE_BUILD_TYPE=Release -DGGML_VULKAN=ON -DGGML_NATIVE=OFF -DGGML_AVX=ON -DGGML_AVX2=ON -DGGML_FMA=ON -DGGML_F16C=ON -DWHISPER_BUILD_TESTS=OFF -DBUILD_SHARED_LIBS=ON
if errorlevel 1 exit /b 1
cmake --build build-vulkan -j 8 --target whisper-cli whisper-server
"@ | Set-Content $bat -Encoding ascii
cmd /c $bat
if ($LASTEXITCODE -ne 0) { throw "Compilation échouée" }
foreach ($f in "whisper-cli.exe", "whisper-server.exe", "whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll", "ggml-vulkan.dll") {
    if (-not (Test-Path "$src\build-vulkan\bin\$f")) { throw "Manquant après compilation : $f" }
}
Write-Host "OK : $src\build-vulkan\bin"
