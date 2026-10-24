# Changelog

Français · English below each version.

## 0.3.1

**Correctifs**
- Les moteurs (llama.cpp, whisper.cpp, traduction comprise) sont arrêtés quand on quitte Verbatim, même en cas de plantage ou de fermeture forcée. Fermer l'app pendant le chargement de Whisper ne laisse plus de processus orphelin qui bloquait les transcriptions au lancement suivant ; les orphelins d'une session précédente sont aussi tués au démarrage.
- Une transcription qui plante ne laisse plus la réunion bloquée en « Préparation de l'audio » ni les suivantes en attente : elle passe en erreur et peut être relancée.
- Le compte-rendu et la traduction affichent le pourcentage de chargement du modèle d'IA.

**Transcription plus rapide**
- Whisper décode en glouton (`-bs 1`) : environ 30 % plus rapide pour un texte quasi identique (2 % d'écart de mots sur un enregistrement de 15 min). Une même phrase répétée plus de deux fois de suite (boucle de décodage) est réduite à deux. Les fragments d'une même phrase sont recollés (6 s au plus) pour garder des lignes de longueur normale.
- Au-delà d'une heure, l'audio est découpé en 3 morceaux aux silences, transcrits en parallèle sur le GPU (1 h : environ 50 s au lieu de 89 s), puis recollés avec leurs horodatages. Sur processeur, une seule instance.
- La préparation de l'audio décode et rééchantillonne en même temps (deux threads) : 15 min en moins d'une seconde, résultat identique. Les builds de développement sont optimisés (opt-level 2).

### English

**Fixes**
- Engines (llama.cpp, whisper.cpp, including translation) are stopped when you quit Verbatim, even after a crash or forced close. Closing the app while Whisper is loading no longer leaves an orphan process that blocked transcriptions on the next launch; orphans from a previous session are also killed at startup.
- A crashing transcription no longer leaves the meeting stuck on "Preparing audio" nor the following ones waiting: it goes to an error state and can be retried.
- The report and the translation show the AI model's loading percentage.

**Faster transcription**
- Whisper decodes greedily (`-bs 1`): about 30% faster for almost identical text (2% word difference on a 15 min recording). A sentence repeated more than twice in a row (decoding loop) is cut back to two. Fragments of the same sentence are merged back (6 s at most) to keep lines at a normal length.
- Beyond one hour, the audio is split into 3 parts at silences, transcribed in parallel on the GPU (1 h: about 50 s instead of 89 s), then stitched back with their timestamps. On CPU, a single instance.
- Audio preparation decodes and resamples at the same time (two threads): 15 min in under a second, identical result. Development builds are optimised (opt-level 2).

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

**Interface**
- Nouvel onglet **Réglages › Général** : langue de l'interface, mises à jour, thème (clair par défaut, sombre ou automatique) et **palettes** qui changent toute l'application : Minuit, Ardoise, Océan, Forêt, Prune, Rose, Ambre, Graphite et deux thèmes à caractère, **Kawaii** (pastels, formes rondes, cœurs et étoiles, chat en fond) et **Gamer** (néon, police monospace). Kawaii et Gamer ont chacun leur animation au clic.

**Divers**
- Le direct charge Whisper dès l'ouverture du panneau d'enregistrement (message de chargement, bouton « Démarrer » grisé tant que Whisper n'est pas prêt) ; Whisper est libéré à l'arrêt.
- Rééchantillonnage de qualité (filtre passe-bas) à l'import et à l'enregistrement : moins de repliement des aigus dans la bande de la voix, donc une transcription plus fidèle.
- Les données (`store.json`) sont écrites de façon atomique.
- Nouveaux modèles Whisper **Large v3 Turbo Q8** (quasi identique à la version complète, recommandé pour le français) et **Large v3 Turbo complet** ; le Turbo compressé passe en Q5.
- Direct : si `whisper-server` s'arrête brutalement, il est relancé automatiquement (3 fois au plus par enregistrement) ; sa sortie est consignée dans `verbatim-whisper-server.log` (dossier temporaire).
- Direct : la capture audio ne fait plus que copier les échantillons, le traitement se fait dans un thread dédié (fin de l'erreur « buffer underrun or overrun »). Les silences et bruits sont écartés par une détection de parole (Earshot) avant l'envoi à Whisper, et les phrases inventées sur du bruit sont filtrées.

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

**Interface**
- New **Settings › General** tab: interface language, updates, theme (light by default, dark or automatic) and **palettes** that restyle the whole app: Midnight, Slate, Ocean, Forest, Plum, Rose, Amber, Graphite and two character themes, **Kawaii** (pastels, round shapes, hearts and stars, a cat in the background) and **Gamer** (neon, monospace font). Kawaii and Gamer each have their own click animation.

**Misc.**
- Live mode loads Whisper as soon as the recording panel opens (loading message, "Start" button greyed out until Whisper is ready); Whisper is released when you stop.
- High-quality resampling (low-pass filter) on import and recording: less aliasing of high frequencies into the voice band, hence a more faithful transcription.
- Data (`store.json`) is written atomically.
- New Whisper models **Large v3 Turbo Q8** (nearly identical to the full version, recommended for French) and **Large v3 Turbo full**; the compressed Turbo is now Q5.
- Live: if `whisper-server` stops abruptly it is restarted automatically (up to 3 times per recording); its output is logged to `verbatim-whisper-server.log` (temp folder).
- Live: audio capture now only copies samples, processing runs on a dedicated thread (no more "buffer underrun or overrun" error). Silence and noise are discarded by speech detection (Earshot) before reaching Whisper, and phrases invented on noise are filtered out.

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
