# Verbatim

> Présentation illustrée : [docs/PRESENTATION.md](docs/PRESENTATION.md) · [Télécharger](https://github.com/Netapick/verbatim/releases/latest)

Application de bureau (Tauri 2 + Rust) pour organiser ses réunions : import audio ou
transcription, compte-rendu, préparations (briefs), glossaire, corbeille.
Identité visuelle, nom, icône et interface originaux.

Données : un fichier `store.json` + les audios copiés, dans le dossier de données de l'app.
LLM optionnel (Réglages) : API compatible OpenAI pour compte-rendu et briefs.

## Enregistrement depuis le PC
Bouton « ● Enregistrer » : micro, son du PC (loopback WASAPI : visio, vidéo…) ou les deux, mixés en mono 16 kHz.
Choix de l'appareil d'entrée et de la sortie à capter (mémorisé), pause/reprise, chronomètre et niveaux en direct ; à l'arrêt, la réunion est créée et transcrite automatiquement.

L'audio est écrit sur disque au fil de l'eau (rien n'est gardé en mémoire). Après un plantage ou une coupure,
l'enregistrement est reconstitué au prochain lancement (perte d'environ 2 s au plus) et ajouté aux réunions.

## Compte-rendu
Généré à partir de la transcription, par ordre de priorité : IA locale (llama.cpp + Qwen 3.5 4B ou 9B, ou Qwen 3.8 27B (Q4 ou allégé), téléchargés dans Réglages ; chargée uniquement pendant la rédaction), puis Ollama (si installé : démarré à la demande, modèle au choix dans Réglages), puis API compatible OpenAI, puis résumé simplifié calculé sur la transcription (phrases réelles, aucun texte inventé). Les longues réunions sont découpées en parties, résumées, puis fusionnées. Le glossaire et la préparation liée sont fournis au modèle.

### Calcul de l'IA de rédaction (CPU / GPU)
Réglages › Compte-rendu : CPU, GPU Vulkan (AMD/NVIDIA/Intel, moteur de 32 Mo, aucune installation) ou GPU ROCm (AMD). Chaque moteur est téléchargé dans `ia/llm`, `ia/llm-vulkan`, `ia/llm-rocm`.
ROCm réutilise le runtime d'un ComfyUI AMD (wheels `rocm_sdk` du venv), détecté automatiquement sur les lecteurs ou à désigner à la main. Repli automatique ROCm → Vulkan → CPU.
Mesure (RX 7800 XT, Qwen 3.5 4B Q4) : CPU 94 / 11 tok/s (lecture / écriture), Vulkan 2566 / 127, ROCm 3673 / 117.

## Écoute
Lecteur audio dans chaque réunion (vitesse 0,75× à 2×) ; un clic sur un passage de la transcription lance la lecture à cet instant, et le passage en cours est surligné.

## Transcription
Whisper local via whisper.cpp v1.9.2 (binaire CPU officiel) : au premier usage, l'app télécharge
le moteur (~8 Mo) et le modèle choisi (tiny 75 Mo, base 142 Mo, small 466 Mo) dans son dossier de données.
L'audio est décodé en Rust (symphonia : mp3, m4a/aac, ogg, flac, wav), converti en 16 kHz mono,
puis transcrit hors-ligne. Le glossaire est transmis à Whisper comme prompt initial.
Modèles au choix dans Réglages (tiny → large v3). Les téléchargements sont reprenables (fichier .part conservé, reprise HTTP Range), leur progression survit aux changements de page, et les modèles restent dans le dossier de données de l'app. La détection de parole (VAD Silero) est activée pour limiter les hallucinations sur les silences.
Calcul : CPU par défaut ; bouton CPU / GPU dans Réglages (GPU = build CUDA officiel de whisper.cpp, cartes NVIDIA uniquement, ~685 Mo à télécharger dans `ia/engine-gpu`).
Limites : pas de diarisation (un seul « Intervenant »), pas de GPU AMD/Intel pour Whisper, Windows x64.

## Emplacement des modèles
Moteurs (whisper.cpp, llama.cpp) et modèles sont rangés dans le dossier `ia` à côté de `verbatim.exe` (à défaut, dans les données de l'app si ce dossier n'est pas inscriptible). Aucun modèle n'est fourni : on les télécharge dans Réglages.
Si des modèles existent déjà (ancien dossier de Verbatim, modèles téléchargés par Ollama, ou tout autre dossier/fichier `.gguf` / `ggml-*.bin` à choisir), Réglages › « Modèles existants » les détecte et les **copie** dans `ia`. Un GGUF copié s'utilise directement avec le moteur de Verbatim, sans Ollama.

## Distribuer l'application
Le dossier `redist/` (moteurs prêts à l'emploi) est embarqué dans l'installateur et utilisé en place : aucun téléchargement de moteur au premier lancement, et plus aucune dépendance à ComfyUI ou à un SDK installé. Les modèles (transcription, rédaction) restent à installer depuis les Réglages.

    ./scripts/build-redist.ps1              # whisper.cpp + VAD + llama.cpp CPU et Vulkan (~170 Mo)
    ./scripts/build-redist.ps1 -WithRocm    # + ROCm AMD RDNA3 : llama.cpp ROCm et runtime minimal (~1,5 Go de plus)
    ./scripts/build-whisper-rocm.ps1 -HipRoot <sdk>   # (une fois) compile whisper.cpp en ROCm, puis build-redist.ps1 -WithRocm -WhisperRocmFrom <...\build-hip\bin>
    npx tauri build                          # installateur NSIS (par utilisateur, dossier `ia` inscriptible)

Whisper en GPU AMD (ROCm) n'a pas de binaire officiel : `build-whisper-rocm.ps1` le compile (≈180 Mo, RDNA3) et le place dans `redist/engine-rocm`, avec le runtime ROCm. Il est choisi automatiquement quand « GPU » est coché et qu'une carte Radeon est détectée.

L'installateur n'affiche aucune question : il installe le pack de base, puis détecte le GPU (`src-tauri/installer/detect-gpu.ps1`) et ajoute le pack ROCm (Radeon RX 7000 / PRO W7000) ou le moteur CUDA de whisper.cpp (NVIDIA, dans `ia/engine-gpu`).

Ordre de recherche d'un moteur : `ia/` (installé par l'utilisateur) puis `redist/` (livré). Pour un installateur léger, construisez `redist` sans `-WithRocm`. Le runtime ROCm livré couvre RDNA3 (gfx110X) ; d'autres cartes AMD utilisent Vulkan.
Les redistribuables tiers (whisper.cpp, llama.cpp, ROCm) ont leurs propres licences : voir les fichiers LICENSE présents dans `redist/`.

## Antivirus
Les moteurs téléchargés (whisper.cpp, llama.cpp : `whisper-cli.exe`, `llama-server.exe`, DLL Vulkan/CUDA/ROCm) ne sont pas signés ; Defender et d'autres antivirus peuvent les supprimer ou les mettre en quarantaine (erreur 5 « accès refusé » au téléchargement, erreur 10054 « connexion fermée » pendant la rédaction). Après extraction, l'app attend l'analyse puis vérifie que les fichiers sont toujours là (`engine::verify_kept`) et affiche la marche à suivre ; si le moteur GPU de rédaction est coupé, elle réessaie une fois sur le CPU. Solution utilisateur : ajouter le dossier d'installation (contenant `ia`) aux exclusions de l'antivirus (Sécurité Windows › Protection contre les virus et menaces › Exclusions), restaurer les fichiers depuis l'historique de protection, puis relancer le téléchargement. Lancer l'app en administrateur n'aide pas.

## Licence et signature
Code sous licence MIT (`LICENSE`). L'installateur est construit et signé par SignPath depuis GitHub Actions (`.github/workflows/release.yml`) : voir `docs/SIGNATURE.md`.

## Code signing policy
Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

- Seul l'installateur construit depuis ce dépôt par GitHub Actions (`.github/workflows/release.yml`) est signé ; chaque signature est approuvée manuellement.
- Rôles : [Netapick](https://github.com/Netapick) est auteur, relecteur et approbateur (authentification multifacteur activée).
- Les moteurs tiers (whisper.cpp, llama.cpp) sont téléchargés tels que publiés par leurs auteurs et ne sont pas signés par ce projet.

## Confidentialité
Verbatim traite l'audio, les transcriptions et les comptes-rendus **uniquement sur l'ordinateur** : rien n'est envoyé en ligne, sauf si vous configurez vous-même une API distante pour la rédaction. Les seules connexions sortantes de l'application : téléchargement des moteurs et modèles (GitHub, Hugging Face) à votre demande, et recherche de mise à jour auprès de l'API GitHub au démarrage (désactivable dans Réglages › Connexions). Aucune donnée personnelle ni télémétrie n'est collectée.

## Mises à jour
Au démarrage, l'app interroge `releases/latest` du dépôt GitHub, télécharge l'installateur (`*_x64-setup.exe`), vérifie son SHA-256 (champ `digest` de l'API), l'exécute en silence dans le même dossier puis se relance (`src-tauri/src/update.rs`). Jamais pendant un enregistrement, une transcription ou une rédaction. Réglages › Connexions permet de désactiver l'installation automatique. Pour publier une version : monter le numéro dans `package.json`, `tauri.conf.json`, `Cargo.toml`, `npx tauri build`, puis `gh release create vX.Y.Z <exe>` (la release doit être la plus récente).

## Identité visuelle
Logo : `python scripts/make-logo.py` (tuile d'application dessinée en raster : dégradés, lueur, reflet ; versions simplifiées en 16/24/32 px) ; exporte `src-tauri/icons/*` (ico + png) et `ui/logo.png`, `ui/favicon.ico`, `ui/favicon.png`.

## Développer
    npm install
    npx tauri dev                    # lancer
    npx tauri build --no-bundle      # exe : src-tauri/target/release/verbatim.exe
    npx tauri build                  # installeurs (nécessite WiX/NSIS)







