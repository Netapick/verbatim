# Changelog

Français · English below each version.

## 0.4.1

**Pastille d'enregistrement**
- Elle se déplace à la souris sans que Windows fasse apparaître une barre de titre par-dessus, et elle ne prend plus le focus : votre visio ou votre présentation reste active quand vous cliquez dessus.
- Elle est détourée en pilule, avec un fond translucide et un contour continu. Il n'y a plus de flou : ceux de Windows laissaient un rectangle autour de la pastille.

**Autres**
- Cette version reprend tout le contenu de la 0.4.0, qui n'a pas été diffusée à cause du défaut de la pastille (voir plus bas).
- Le pack Whisper Vulkan de l'installateur est hébergé par la release de cette version.

### English

**Recording bar**
- It moves with the mouse without Windows drawing a title bar over it, and it no longer takes focus: your video call or presentation stays active when you click it.
- It is cut out as a pill, with a translucent background and a continuous outline. There is no blur any more: Windows' blur effects left a rectangle around the bar.

**Other**
- This version includes everything in 0.4.0, which was not distributed because of the bar's defect (see below).
- The installer's Whisper Vulkan pack is hosted by this version's release.

## 0.4.0

**Séparation des intervenants**
- Chaque passage est attribué à son intervenant (« Intervenant 1 », « Intervenant 2 »…), calculé sur le processeur après la transcription ; le nombre d'intervenants est déterminé automatiquement. Les modèles (environ 34 Mo) sont téléchargés avec le moteur Whisper, ou depuis Réglages › Transcription.
- Quand une personne se présente (« Bonjour, je suis Marie »), sa voix est retenue et elle est nommée tout au long de la réunion. Les voix qui ne se présentent pas gardent leur numéro.
- Couleur par intervenant, clic sur un nom pour le renommer (partout, traductions comprises), bouton « Identifier les intervenants » pour une réunion déjà transcrite.

**Whisper en Vulkan**
- La transcription (et le direct) utilise le GPU sur les cartes AMD d'avant la série RX 7000 et sur Intel, où ROCm n'existe pas. Le moteur (17 Mo à télécharger) est téléchargé par l'installateur quand une telle carte est détectée ; il s'ajoute au choix « Vulkan » de Réglages › Transcription. Mesuré sur une RX 7800 XT : environ 7 fois plus rapide que le processeur (120 s d'audio en 8,6 s contre 59,6 s, texte identique).

**Mise à jour intégrée**
- Un bandeau en haut annonce la nouvelle version (avec les notes). Une fenêtre centrée sur fond flouté montre le téléchargement (progression), puis l'installation : l'application se ferme et redémarre toute seule. Avec « Installer automatiquement », la version est téléchargée en arrière-plan et rien n'est installé avant votre clic sur « Mettre à jour ».
- La mise à jour est refusée (bouton grisé, raison affichée) pendant un enregistrement, une transcription, une rédaction, une traduction ou un téléchargement.
- Les mises à jour sont signées (minisign) et vérifiées avant l'installation.

**Autres**
- Écran de démarrage : logo animé et barre de chargement. Pendant ce temps, Whisper est chargé (il reste prêt pour l'enregistrement en direct jusqu'à la fermeture) et la séparation des intervenants est chargée en mémoire : la première transcription démarre plus vite. Whisper est libéré pendant la rédaction d'un compte-rendu, pour laisser la mémoire à l'IA locale, et se recharge à l'ouverture du panneau d'enregistrement.
- L'IA locale du compte-rendu reste chargée après sa première utilisation (les comptes-rendus suivants démarrent tout de suite) ; elle est arrêtée si le modèle ou le matériel change, à la suppression d'un modèle, et tous les moteurs sont arrêtés à la fermeture de l'application.
- La séparation des intervenants affiche son avancement en pourcentage (estimé d'après la vitesse de la séparation précédente).
- Réunions : bouton pour renommer directement depuis la liste ; fenêtre au démarrage d'un enregistrement rappelant de dire « Bonjour, je suis… » ; l'application s'ouvre maximisée et le contenu occupe toute la largeur.
- Glisser-déposer : déposez des fichiers audio (ou une transcription) sur la fenêtre pour les importer.
- Enregistrement en arrière-plan : pendant un enregistrement, la fenêtre se masque (plus de bouton dans la barre des tâches, seule reste l'icône de la zone de notification) et une pastille translucide, toujours visible (en haut de l'écran, déplaçable), offre Pause / Reprendre, Terminer et le retour à la fenêtre. L'icône de notification donne les mêmes actions (clic gauche : afficher).
- La case « Transcription en direct » est décochée à chaque ouverture du panneau d'enregistrement.
- Les modèles de séparation des intervenants (34 Mo) sont téléchargés seuls au démarrage quand Whisper est déjà installé (une seule fois : supprimés à la main, ils ne reviennent pas).
- Sous « Réunions », des cartes proposent d'installer la séparation des intervenants et de choisir un modèle de compte-rendu quand l'un des deux manque (Whisper étant installé).
- La version affichée est `v0.4.0` ; les versions de développement ajoutent l'horodatage du build (`v0.4.0-20261025-1130`).
- Compte-rendu : si le modèle est coupé par la limite de longueur, la rédaction est relancée avec une limite plus large ; l'erreur indique la cause.

### English

**Speaker separation**
- Each passage is assigned to its speaker ("Speaker 1", "Speaker 2"…), computed on the processor after transcription; the number of speakers is determined automatically. The models (about 34 MB) are downloaded with the Whisper engine, or from Settings › Transcription.
- When someone introduces themselves ("Hello, I'm Marie"), their voice is remembered and they are named throughout the meeting. Voices that do not introduce themselves keep their number.
- A colour per speaker, click a name to rename it (everywhere, translations included), an "Identify speakers" button for an already transcribed meeting.

**Whisper on Vulkan**
- Transcription (and live) uses the GPU on AMD cards older than the RX 7000 series and on Intel, where ROCm does not exist. The engine (17 MB to download) is downloaded by the installer when such a card is detected; it adds the "Vulkan" choice in Settings › Transcription. Measured on an RX 7800 XT: about 7 times faster than the processor (120 s of audio in 8.6 s versus 59.6 s, identical text).

**Built-in update**
- A banner at the top announces the new version (with its notes). A centred window on a blurred background shows the download (progress), then the installation: the application closes and restarts by itself. With "Install automatically", the version is downloaded in the background and nothing is installed until you click "Update".
- Updating is refused (greyed-out button, reason shown) during a recording, a transcription, report writing, a translation or a download.
- Updates are signed (minisign) and verified before installation.

**Other**
- Start-up screen: animated logo and loading bar. Meanwhile, Whisper is loaded (it stays ready for live recording until the application closes) and speaker separation is loaded into memory: the first transcription starts faster. Whisper is released while a report is being written, to leave the memory to the local AI, and reloads when the recording panel opens.
- The local report AI stays loaded after its first use (later reports start right away); it is stopped if the model or hardware changes, when a model is deleted, and all engines are stopped when the application closes.
- Speaker separation shows its progress as a percentage (estimated from the speed of the previous separation).
- Meetings: a button to rename straight from the list; a window when starting a recording reminding to say “Hello, I'm…”; the application opens maximised and the content uses the full width.
- Drag and drop: drop audio files (or a transcript) on the window to import them.
- Background recording: while recording, the window is hidden (no taskbar button, only the notification-area icon remains) and a translucent, always-visible bar (top of the screen, draggable) offers Pause / Resume, Stop and a way back to the window. The notification icon gives the same actions (left click: show).
- The “Live transcription” checkbox is unchecked every time the recording panel opens.
- The speaker-separation models (34 MB) are downloaded automatically at startup when Whisper is already installed (once only: if you delete them by hand, they do not come back).
- Under “Meetings”, cards offer to install speaker separation and to choose a report model when either is missing (once Whisper is installed).
- The displayed version is `v0.4.0`; development versions add the build timestamp (`v0.4.0-20261025-1130`).
- Report: if the model is cut off by the length limit, writing is retried with a larger limit; the error states the cause.

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
