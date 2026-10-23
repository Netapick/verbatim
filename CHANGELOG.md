# Changelog

Français · English below each version.

## 0.2.3

**Interface en anglais**
- Verbatim s'affiche en français ou en anglais. Choix dans Réglages › Connexions ; par défaut, la langue choisie dans l'installateur, puis celle de Windows.
- Seuls les textes de l'interface sont traduits : vos transcriptions et comptes-rendus restent dans la langue de l'audio.

**Lecture**
- Correction du décalage entre le son et le texte surligné sur les audios importés en mp3 : l'app garde une copie WAV (celle qui est transcrite) pour la lecture, où le positionnement est exact. S'applique aux nouvelles transcriptions ; une réunion déjà transcrite garde sa lecture d'origine.

**Installateur**
- Installateur bilingue (français / anglais) : écran de choix de la langue, messages et journal d'installation traduits.
- La langue choisie est reprise par l'application au premier lancement.
- Accents corrects dans la détection de la carte graphique et les téléchargements.

**Documentation**
- README, présentation, politique de confidentialité et documentation technique disponibles en anglais.
- Nouvelle section « Performances et consommation » (vitesse, mémoire, mémoire vidéo, processeur).
- Licence MIT, politique de signature de code et de confidentialité.

*Cet installateur n'est pas encore signé : SmartScreen ou l'antivirus peut l'avertir.*

### English

**English interface**
- Verbatim can be displayed in French or English. Choose in Settings › Connections; by default, the language picked in the installer, then Windows' language.
- Only interface texts are translated: your transcripts and reports stay in the audio's language.

**Playback**
- Fixed the offset between sound and highlighted text on imported mp3 files: the app keeps a WAV copy (the one that is transcribed) for playback, where seeking is exact. Applies to new transcriptions; a meeting already transcribed keeps its original playback.

**Installer**
- Bilingual installer (French / English): language selection screen, translated messages and installation log.
- The chosen language is picked up by the app at first launch.
- Correct accents in graphics card detection and downloads.

**Documentation**
- README, presentation, privacy policy and technical documentation available in English.
- New "Performance and resource usage" section (speed, memory, video memory, processor).
- MIT license, code signing and privacy policies.

*This installer is not signed yet: SmartScreen or your antivirus may warn about it.*

## 0.2.2

**NVIDIA / CUDA**
- Moteur CUDA pour la rédaction (llama.cpp), installé automatiquement sur carte NVIDIA et proposé dans Réglages › Compte-rendu (« Forcer CUDA »). « Forcer ROCm » n'apparaît plus que sur carte AMD.
- Mode Automatique : CUDA, puis ROCm, puis Vulkan, puis processeur.

**Réglages**
- Les modèles Whisper non téléchargés sont verrouillés, comme ceux du compte-rendu.
- Les modèles trop volumineux pour la mémoire vidéo sont affichés en rouge et barrés, avec un message.
- Bouton « Arrêter » pour interrompre un téléchargement de modèle ou de moteur (il reprend ensuite).
- Encart « Antivirus : ajoutez une exclusion » avec le chemin du dossier `ia`.

**Antivirus**
- Après l'installation d'un moteur, Verbatim vérifie que les fichiers n'ont pas été supprimés ou mis en quarantaine et explique quoi faire.
- Si le moteur GPU de rédaction est coupé, nouvel essai automatique sur le processeur.

**Installateur et divers**
- Message d'information sur l'exclusion antivirus du dossier `ia` ; accents corrigés dans le journal.
- Moteur CUDA de rédaction installé sur NVIDIA (≈ 630 Mo de plus).
- Plus de fenêtre cmd qui clignote à l'ouverture des Réglages.

### English

- **NVIDIA / CUDA:** CUDA engine for report writing, installed automatically on NVIDIA cards and selectable in Settings › Report ("Force CUDA"); "Force ROCm" only shows on AMD. Automatic mode: CUDA, ROCm, Vulkan, then CPU.
- **Settings:** models not yet downloaded are locked; models too large for the video memory are shown in red and struck through; "Stop" button for model and engine downloads (they resume later); antivirus exclusion notice with the `ia` folder path.
- **Antivirus:** after an engine install, Verbatim checks that files were not deleted or quarantined and explains what to do; if the GPU writing engine is cut off, one automatic retry on the CPU.
- **Installer and misc.:** antivirus exclusion info message, accents fixed in the log, CUDA writing engine installed on NVIDIA (≈ 630 MB more), no more flashing cmd window when opening Settings.
