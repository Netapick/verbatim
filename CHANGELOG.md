# Changelog

Français · English below each version.

## 0.3.0

**Compte-rendu**
- La langue de rédaction se choisit : dans Réglages › Compte-rendu (par défaut) et, réunion par réunion, à côté du bouton « Générer ». « Automatique » rédige dans la langue de la réunion. Les titres du compte-rendu et son export suivent la langue choisie.

**Traduction de la transcription**
- Nouveau menu « Traduire en… » dans la carte Transcription (14 langues). La traduction est faite par l'IA locale, segment par segment : les horodatages sont conservés, le lecteur et le surlignage fonctionnent aussi sur la traduction. Une traduction par langue, conservée avec la réunion.
- Nouveaux modèles **TranslateGemma 4B et 12B** (Google, 55 langues), exécutés par llama.cpp. À défaut, le modèle du compte-rendu est utilisé.
- Nouvel onglet **Réglages › Traduction** : calcul (automatique, ou processeur seul), modèle, et dossier des modèles (`ia	ranslation-models`).
- Pendant la traduction : pourcentage, modèle, matériel utilisé (CPU ou GPU) et bouton « Arrêter ».
- Les exports .txt, .srt, .vtt et .json suivent la langue affichée.

**Transcription en direct**
- Case « Transcription en direct » dans le panneau d'enregistrement, sous les niveaux du micro et du son du PC. Le texte s'affiche pendant la réunion, avec une latence de quelques secondes ; une traduction en direct peut s'afficher sous chaque phrase. Le point rouge de l'enregistrement pulse pendant le direct.
- À l'arrêt, la transcription complète du fichier est refaite comme d'habitude (meilleure qualité) : le texte du direct est provisoire.
- Sur carte **AMD Radeon RX 7000 / PRO W7000**, le direct utilise un `whisper-server` compilé en ROCm (une inférence de 10 s en 0,16 s au lieu de 1,2 s sur processeur) et accepte tous les modèles Whisper ; sur processeur, modèles Tiny à Small.

**Installateur**
- Le pack ROCm est mis à jour (ajout de `whisper-server`). Une mise à jour depuis une version précédente ne télécharge que ce petit fichier (≈ 0,5 Mo) ; une installation neuve télécharge le pack complet (≈ 414 Mo). Les packs sont désormais sur la release v0.3.0.

**Divers**
- Les données (`store.json`) sont écrites de façon atomique.

### English

**Report**
- The writing language can be chosen: in Settings › Report (default) and, meeting by meeting, next to the "Generate" button. "Automatic" writes in the meeting's language. The report's headings and its export follow the chosen language.

**Transcript translation**
- New "Translate into…" menu in the Transcription card (14 languages). The translation is done by the local AI, segment by segment: timestamps are kept, so the player and highlighting also work on the translation. One translation per language, stored with the meeting.
- New **TranslateGemma 4B and 12B** models (Google, 55 languages), run by llama.cpp. Otherwise the report model is used.
- New **Settings › Translation** tab: compute (automatic, or processor only), model, and models folder (`ia	ranslation-models`).
- During translation: percentage, model, hardware used (CPU or GPU) and a "Stop" button.
- .txt, .srt, .vtt and .json exports follow the displayed language.

**Live transcription**
- "Live transcription" checkbox in the recording panel, under the microphone and PC sound levels. Text appears during the meeting with a few seconds of delay; a live translation can appear under each sentence. The red recording dot pulses during live mode.
- When you stop, the full transcription of the file is redone as usual (better quality): live text is provisional.
- On an **AMD Radeon RX 7000 / PRO W7000** card, live mode uses a `whisper-server` built for ROCm (a 10 s inference in 0.16 s instead of 1.2 s on the processor) and accepts every Whisper model; on the processor, Tiny to Small models.

**Installer**
- The ROCm pack is updated (adds `whisper-server`). Updating from a previous version only downloads this small file (≈ 0.5 MB); a fresh install downloads the full pack (≈ 414 MB). Packs now live on the v0.3.0 release.

**Misc.**
- Data (`store.json`) is written atomically.

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
