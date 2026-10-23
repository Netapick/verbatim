# Documentation technique

Français · [English](DEVELOPMENT.en.md)

Verbatim est une application de bureau **Tauri 2 + Rust** (interface HTML/JS sans framework dans `ui/`), pour Windows x64.

## Organisation du code

| Dossier / fichier | Rôle |
|---|---|
| `ui/` | Interface : `app.js` (vues), `i18n.js` (traduction), `style.css`, `index.html` |
| `src-tauri/src/main.rs` | Données (réunions, préparations, glossaire, réglages), commandes Tauri |
| `src-tauri/src/engine.rs` | Transcription : whisper.cpp, décodage audio (symphonia), téléchargements reprenables, détection GPU |
| `src-tauri/src/llm.rs` | Rédaction : llama.cpp (serveur local), Ollama, API distante, résumé simplifié |
| `src-tauri/src/recorder.rs` | Enregistrement micro et son du PC (cpal, WASAPI loopback) |
| `src-tauri/src/models.rs` | Détection et copie de modèles existants |
| `src-tauri/src/update.rs` | Mise à jour automatique |
| `src-tauri/installer/` | Hooks NSIS et scripts PowerShell de l'installateur |
| `scripts/` | Construction des moteurs et des packs, logo |

Les données sont dans un fichier `store.json` et les audios copiés, dans le dossier de données de l'application.

## Enregistrement

Micro, son du PC (boucle WASAPI : visio, vidéo) ou les deux, mixés en mono 16 kHz, avec choix des appareils (mémorisé), pause/reprise, chronomètre et niveaux en direct. L'audio est écrit sur disque au fil de l'eau : après un plantage ou une coupure, l'enregistrement est reconstitué au prochain lancement (perte d'environ 2 s au plus).

## Transcription

- whisper.cpp v1.9.2 (binaire officiel CPU) ; l'audio est décodé en Rust (symphonia : mp3, m4a/aac, ogg, flac, wav), converti en 16 kHz mono et transcrit hors ligne. Le glossaire est transmis à Whisper comme prompt initial. La détection de parole (VAD Silero) limite les hallucinations sur les silences.
- Modèles : tiny, base, small, medium-q5_0, large-v3-turbo-q5_0, large-v3-q5_0, téléchargés depuis Hugging Face (`ggerganov/whisper.cpp`).
- Téléchargements reprenables : fichier `.part` conservé, reprise HTTP Range, progression conservée entre les pages ; bouton d'arrêt (`download_cancel`).
- Calcul : mode « Automatique » (GPU si disponible, repli sur le CPU) ou « CPU uniquement ». NVIDIA : build CUDA officiel de whisper.cpp (b5454, ≈ 685 Mo, `ia/engine-gpu`). AMD : whisper.cpp n'a pas de binaire officiel, donc `scripts/build-whisper-rocm.ps1` le compile en HIP/ROCm pour RDNA3 (≈ 180 Mo, `redist/engine-rocm`).

## Rédaction du compte-rendu

Ordre de priorité : IA locale (llama.cpp + Qwen, chargée uniquement pendant la rédaction), puis Ollama (démarré à la demande), puis API compatible OpenAI, puis résumé simplifié calculé sur la transcription (phrases réelles, aucun texte inventé). Les longues réunions sont découpées en parties, résumées, puis fusionnées. Le glossaire et la préparation liée sont fournis au modèle.

### Moteurs de calcul de la rédaction

| Moteur | Dossier | Notes |
|---|---|---|
| CPU | `ia/llm` | Compatible partout |
| Vulkan | `ia/llm-vulkan` | AMD, NVIDIA, Intel ; ≈ 32 Mo |
| ROCm | `ia/llm-rocm` | AMD RDNA3 ; runtime livré dans `redist/rocm-runtime` ou désigné à la main |
| CUDA | `ia/llm-cuda` | NVIDIA ; deux archives llama.cpp (moteur + DLL CUDA) |

Mode automatique : CUDA, puis ROCm, puis Vulkan, puis CPU ; si le moteur GPU est coupé en cours de rédaction, un nouvel essai est fait sur le CPU. Un modèle plus gros que la mémoire vidéo disponible est signalé dans les Réglages (lecture du registre Windows, `engine::gpu_vram`).

Mesures de vitesse et de consommation (RAM, VRAM, processeur) : voir la section « Performances et consommation » du [README](../README.md). Les graphiques sont générés par `scripts/make-benchmark-charts.py` (sortie dans `docs/benchmarks/`, avec les données brutes `results.json`).

## Moteurs et modèles : emplacements

Les moteurs et modèles sont rangés dans `ia/` à côté de `verbatim.exe` (à défaut, dans les données de l'application si ce dossier n'est pas inscriptible). Ordre de recherche d'un moteur : `ia/` (installé par l'utilisateur), puis `redist/` (livré). Réglages › Modèles détecte les modèles existants (ancien dossier de Verbatim, modèles d'Ollama, `.gguf` / `ggml-*.bin` à choisir) et les **copie** dans `ia`.

## Installateur et distribution

L'installateur est un NSIS léger (≈ 3 Mo, par utilisateur, français et anglais). Les moteurs sont téléchargés pendant l'installation, sans aucune question :

1. Pack de base (≈ 61 Mo : whisper.cpp, VAD, llama.cpp CPU et Vulkan).
2. Détection de la carte graphique (`src-tauri/installer/detect-gpu.ps1`), puis pack ROCm (Radeon RX 7000 / PRO W7000, ≈ 430 Mo) ou moteurs CUDA (NVIDIA : Whisper b5454 et llama.cpp b11700 + DLL CUDA).
3. Écriture de la langue choisie dans `lang.txt`, reprise par l'application au premier démarrage.

Les packs sont des zips publiés sur la release `v0.1.0` du dépôt ; `packs.nsh` (URL, taille, SHA-256) est généré par `scripts/make-packs.ps1`. Journal de l'installation : `redist-install.log`.

```
./scripts/build-redist.ps1              # whisper.cpp + VAD + llama.cpp CPU et Vulkan
./scripts/build-redist.ps1 -WithRocm    # + ROCm AMD RDNA3 (runtime minimal)
./scripts/build-whisper-rocm.ps1 -HipRoot <sdk>   # compile whisper.cpp en ROCm (une fois)
./scripts/make-packs.ps1 -BaseUrl <url> # zips des packs + packs.nsh
npx tauri build                         # installateur NSIS
```

## Mises à jour automatiques

Au démarrage, l'app interroge `releases/latest` du dépôt GitHub, télécharge l'installateur (`*_x64-setup.exe`), vérifie son SHA-256 (champ `digest` de l'API), l'exécute en silence dans le même dossier puis se relance (`src-tauri/src/update.rs`). Jamais pendant un enregistrement, une transcription ou une rédaction. Désactivable dans Réglages › Connexions.

## Antivirus

Les moteurs tiers ne sont pas signés et peuvent être supprimés par un antivirus. Après chaque extraction, `engine::verify_kept` attend l'analyse puis vérifie que les fichiers sont toujours là, et `engine::fs_err` traduit les refus d'accès en message explicite. Voir la section « Si votre antivirus bloque un moteur » du [README](../README.md).

## Traduction de l'interface

Les textes de l'interface sont écrits en français dans le code. `ui/i18n.js` contient un dictionnaire français → anglais et des motifs pour les messages à valeurs variables, appliqués à l'affichage (nœuds texte et attributs). Pour traduire un nouveau texte, ajoutez sa paire dans `EN` (clé = texte français exact) ou un motif dans `P`. Le choix de langue est dans Réglages › Connexions ; à défaut, la langue de l'installateur (`lang.txt`), puis celle du système.

## Construire

```
npm install
npx tauri dev                    # lancer en développement
npx tauri build --no-bundle      # exe seul : src-tauri/target/release/verbatim.exe
npx tauri build                  # installateur (nécessite NSIS)
```

## Publier une version

1. Monter le numéro dans `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` et `src-tauri/Cargo.lock`, ajouter une entrée à `CHANGELOG.md`, puis commiter.
2. Pousser un tag `vX.Y.Z` : le workflow `.github/workflows/release.yml` construit l'installateur, le fait signer par SignPath et publie la release (voir [SIGNATURE.md](SIGNATURE.md)). La release doit être la plus récente pour que la mise à jour automatique la voie.

## Identité visuelle

Logo : `python scripts/make-logo.py` (tuile d'application dessinée en raster : dégradés, lueur, reflet ; versions simplifiées en 16/24/32 px) ; exporte `src-tauri/icons/*` (ico + png) et `ui/logo.png`, `ui/favicon.ico`, `ui/favicon.png`.
