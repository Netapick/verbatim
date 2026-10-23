// Traduction de l'interface : le texte de l'app est écrit en français ; en anglais, chaque texte affiché est
// remplacé à la volée d'après ce dictionnaire (texte entier, ou motif pour les messages à valeurs variables).
(() => {
  const norm = (s) => s.replace(/\s+/g, " ").trim();

  // ── Dictionnaire (texte français exact, espaces normalisés → anglais) ──
  const EN = Object.fromEntries([
    // Navigation, boutons communs
    ["Réunions", "Meetings"], ["Préparations", "Preparations"], ["Glossaire", "Glossary"], ["Corbeille", "Trash"], ["Réglages", "Settings"],
    ["Tout reste sur cet ordinateur : audio, transcriptions et IA locale.", "Everything stays on this computer: audio, transcripts and local AI."],
    ["Basculer thème clair / sombre", "Switch light / dark theme"], ["Navigation principale", "Main navigation"],
    ["Annuler", "Cancel"], ["Supprimer", "Delete"], ["Valider", "OK"], ["Mettre à la corbeille", "Move to trash"], ["Enregistrer", "Save"],
    ["Renommer", "Rename"], ["Copier", "Copy"], ["Restaurer", "Restore"], ["Retirer", "Remove"], ["Télécharger", "Download"],
    ["Reprendre", "Resume"], ["Installé", "Installed"], ["Téléchargement…", "Downloading…"], ["Rechercher", "Search"], ["Recherche…", "Searching…"],
    ["Arrêter", "Stop"], ["Arrêt…", "Stopping…"],
    // Téléchargements
    ["Téléchargement en cours…", "Download in progress…"],
    ["Vous pouvez changer de page ou fermer l'app : le téléchargement reprendra là où il s'est arrêté.", "You can change page or close the app: the download will resume where it stopped."],
    ["Téléchargement interrompu", "Download interrupted"], ["Moteur de transcription non installé", "Transcription engine not installed"],
    ["Reprendre le téléchargement", "Resume download"], ["Installer Whisper", "Install Whisper"],
    ["Téléchargement arrêté — vous pourrez le reprendre.", "Download stopped — you can resume it later."], ["Téléchargement annulé.", "Download cancelled."],
    ["Moteur Whisper prêt", "Whisper engine ready"],
    ["Moteur", "Engine"], ["Détection de parole", "Speech detection"], ["Modèle", "Model"], ["Moteur IA", "AI engine"], ["Modèle IA", "AI model"],
    ["Copie", "Copy"], ["Moteur GPU", "GPU engine"], ["Téléchargement", "Download"],
    // Enregistrement
    ["Nouvel enregistrement", "New recording"], ["Microphone", "Microphone"], ["Sensibilité du micro", "Microphone sensitivity"],
    ["Son du PC (visio, vidéo…)", "PC sound (video calls, videos…)"], ["Sensibilité du son du PC", "PC sound sensitivity"], ["● Démarrer", "● Start"],
    ["Pour une visioconférence, cochez les deux : votre voix et celle de vos interlocuteurs sont enregistrées. Pour « Son du PC », choisissez la sortie sur laquelle vous écoutez (casque, haut-parleurs).",
      "For a video call, tick both: your voice and your contacts' voices are recorded. For “PC sound”, choose the output you listen on (headset, speakers)."],
    ["Par défaut", "Default"], ["En pause", "Paused"], ["Enregistrement en cours", "Recording in progress"], ["Micro", "Mic"], ["PC", "PC"],
    ["Sensibilité", "Sensitivity"], ["Titre (facultatif)", "Title (optional)"], ["▶ Reprendre", "▶ Resume"], ["❚❚ Pause", "❚❚ Pause"], ["■ Terminer", "■ Stop"],
    ["Enregistrement ajouté — transcription lancée", "Recording added — transcription started"],
    // Réunions
    ["Importez un enregistrement ou une transcription existante.", "Import a recording or an existing transcript."],
    ["● Enregistrer", "● Record"], ["+ Importer un audio", "+ Import audio"], ["Importer une transcription", "Import a transcript"],
    ["Aucune réunion pour le moment.", "No meetings yet."], ["Prête", "Ready"], ["Échec", "Failed"], ["Moteur requis", "Engine required"],
    ["En attente", "Queued"], ["Préparation de l'audio", "Preparing audio"], ["Transcription en cours", "Transcribing"], ["Moteur Whisper requis", "Whisper engine required"],
    ["← Réunions", "← Meetings"], ["Audio · transcription Whisper locale", "Audio · local Whisper transcription"], ["Transcription importée", "Imported transcript"],
    ["Vitesse de lecture", "Playback speed"], ["Impossible de lire cet audio (format non pris en charge ?)", "Cannot play this audio (unsupported format?)"],
    ["La transcription a échoué", "Transcription failed"], ["Réessayer", "Retry"],
    ["Générer le compte-rendu", "Generate the report"], ["Régénérer le compte-rendu", "Regenerate the report"],
    ["Aucune préparation liée", "No linked preparation"], ["Exporter…", "Export…"], ["Transcription", "Transcription"],
    ["Texte (.txt)", "Text (.txt)"], ["Sous-titres (.srt)", "Subtitles (.srt)"], ["Sous-titres (.vtt)", "Subtitles (.vtt)"], ["Données (.json)", "Data (.json)"],
    ["Compte-rendu", "Report"], ["Compte-rendu (.md)", "Report (.md)"], ["Enregistrement", "Recording"], ["Audio", "Audio"],
    ["Génération du compte-rendu…", "Generating the report…"],
    ["Avec l'IA locale, une longue réunion peut prendre plusieurs minutes. Vous pouvez quitter cette page : la génération continue.", "With the local AI, a long meeting can take several minutes. You can leave this page: generation continues."],
    ["Échec de la génération", "Generation failed"], ["Résumé", "Summary"], ["Points abordés", "Topics covered"], ["Décisions", "Decisions"],
    ["Actions", "Actions"], ["Prochaines étapes", "Next steps"], ["Écouter à partir d'ici", "Listen from here"], ["Intervenant", "Speaker"],
    ["Placer cette réunion dans la corbeille ?", "Move this meeting to the trash?"], ["Nouveau titre", "New title"],
    ["Placée dans la corbeille", "Moved to the trash"], ["Exporté", "Exported"],
    ["Résumé automatique simplifié.", "Simplified automatic summary."], ["Ouvrir les Réglages", "Open Settings"], ["IA", "AI"],
    ["Résumé automatique simplifié (aucune IA de rédaction installée). Installez un modèle dans Réglages › Compte-rendu pour obtenir un compte-rendu rédigé.",
      "Simplified automatic summary (no writing AI installed). Install a model in Settings › Report to get a written report."],
    ["Le moteur d'IA locale (llama.cpp) n'est pas installé : téléchargez d'abord un modèle dans Réglages. Résumé simplifié affiché.",
      "The local AI engine (llama.cpp) is not installed: download a model in Settings first. Simplified summary shown."],
    ["Le fichier du modèle est introuvable. Résumé simplifié affiché.", "The model file cannot be found. Simplified summary shown."],
    // Avancement (côté moteur)
    ["Rédaction du compte-rendu…", "Writing the report…"], ["Fusion des parties…", "Merging parts…"], ["Démarrage d'Ollama…", "Starting Ollama…"],
    ["Chargement du modèle (la première fois peut être long)…", "Loading the model (the first time can be slow)…"],
    ["Chargement de l'IA locale (gros modèle : plusieurs minutes possibles)…", "Loading the local AI (large model: may take several minutes)…"],
    ["Moteur GPU interrompu : nouvel essai sur le processeur…", "GPU engine interrupted: retrying on the processor…"],
    ["Chargement de l'IA locale…", "Loading the local AI…"], ["Démarrage…", "Starting…"], ["Ollama (matériel géré par Ollama)", "Ollama (hardware managed by Ollama)"],
    // Préparations
    ["Préparez chaque réunion : objectif, ordre du jour et questions à poser.", "Prepare each meeting: objective, agenda and questions to ask."],
    ["+ Nouvelle préparation", "+ New preparation"], ["Nouvelle préparation", "New preparation"], ["Sujet de la réunion", "Meeting subject"],
    ["Ex. Revue de budget T3", "E.g. Q3 budget review"], ["Type de réunion", "Meeting type"], ["Gérer les types", "Manage types"],
    ["Thèmes à aborder", "Topics to cover"], ["(séparés par des virgules, facultatif)", "(comma-separated, optional)"],
    ["budget, risques, planning", "budget, risks, schedule"], ["Générer la préparation", "Generate the preparation"],
    ["Génération en cours…", "Generating…"], ["Indiquez un sujet", "Enter a subject"], ["Aucune préparation pour l'instant.", "No preparations yet."],
    ["Renseignez un sujet ci-dessus pour générer un plan de réunion.", "Enter a subject above to generate a meeting plan."],
    ["Types de réunion", "Meeting types"],
    ["Le nom apparaît dans les listes. La consigne (facultative) guide l'IA, par exemple « Tu prépares un comité budgétaire, centré sur les chiffres ».",
      "The name appears in the lists. The instruction (optional) guides the AI, for example “You are preparing a budget committee, focused on the figures”."],
    ["+ Ajouter un type", "+ Add a type"], ["Nom du type", "Type name"], ["Consigne pour l'IA (facultatif)", "Instruction for the AI (optional)"],
    ["Supprimer ce type", "Delete this type"], ["Il faut au moins un type", "At least one type is required"],
    ["← Préparations", "← Preparations"], ["Objectif", "Objective"], ["Que doit-on avoir obtenu à la fin de la réunion ?", "What should have been achieved by the end of the meeting?"],
    ["Ordre du jour", "Agenda"], ["+ Point", "+ Item"], ["Questions à poser", "Questions to ask"], ["+ Question", "+ Question"], ["Réunions liées", "Linked meetings"],
    ["Modifications non enregistrées", "Unsaved changes"], ["Point à traiter", "Item to cover"], ["min", "min"], ["Question", "Question"],
    ["Questions", "Questions"], ["Enregistré", "Saved"], ["Copié dans le presse-papiers", "Copied to clipboard"],
    ["Placer cette préparation dans la corbeille ?", "Move this preparation to the trash?"],
    ["Réunion générale", "General meeting"], ["Entretien 1:1", "1:1 meeting"], ["Point projet", "Project update"], ["Comité de pilotage", "Steering committee"],
    ["Brainstorming", "Brainstorming"],
    // Glossaire, corbeille
    ["Sigles et termes métier propres à votre organisation.", "Acronyms and business terms specific to your organization."],
    ["+ Terme", "+ Term"], ["Terme", "Term"], ["Définition", "Definition"], ["Glossaire enregistré", "Glossary saved"],
    ["Les éléments supprimés définitivement ne sont pas récupérables.", "Permanently deleted items cannot be recovered."],
    ["Réunion", "Meeting"], ["Brief", "Brief"], ["La corbeille est vide.", "The trash is empty."],
    ["Supprimer définitivement ? Cette action est irréversible.", "Delete permanently? This cannot be undone."],
    // Réglages
    ["Tout est enregistré automatiquement, sauf la connexion API.", "Everything is saved automatically, except the API connection."],
    ["Connexions", "Connections"], ["Modèles", "Models"],
    ["Antivirus : ajoutez une exclusion", "Antivirus: add an exclusion"],
    ["Les moteurs d'IA (whisper.cpp, llama.cpp) ne sont pas signés : votre antivirus peut les supprimer ou les bloquer (erreurs « accès refusé » ou « connexion fermée »). Pour l'éviter, ajoutez ce dossier aux exclusions de votre antivirus :",
      "The AI engines (whisper.cpp, llama.cpp) are not signed: your antivirus may delete or block them (“access denied” or “connection closed” errors). To avoid this, add this folder to your antivirus exclusions:"],
    ["Copier le chemin", "Copy path"], ["Chemin copié", "Path copied"],
    ["Windows Defender : Sécurité Windows › Protection contre les virus et menaces › Gérer les paramètres › Exclusions › Ajouter une exclusion › Dossier.",
      "Windows Defender: Windows Security › Virus & threat protection › Manage settings › Exclusions › Add an exclusion › Folder."],
    ["Langue et calcul", "Language and computation"], ["Langue parlée", "Spoken language"],
    ["Français", "French"], ["Anglais", "English"], ["Espagnol", "Spanish"], ["Allemand", "German"], ["Italien", "Italian"], ["Détection auto", "Auto-detect"],
    ["Modèle de transcription", "Transcription model"],
    ["Les modèles « compressés » sont presque aussi précis que les complets, pour moitié moins d'espace. Plus le modèle est gros, plus c'est lent.",
      "“Compressed” models are almost as accurate as full ones, for half the space. The bigger the model, the slower it is."],
    ["Calcul de la rédaction", "Writing computation"], ["IA de rédaction", "Writing AI"],
    ["Un modèle installé sur cet ordinateur rédige le compte-rendu, sans rien envoyer en ligne. Sans modèle local ni API, seul un résumé simplifié est produit.",
      "A model installed on this computer writes the report, without sending anything online. Without a local model or API, only a simplified summary is produced."],
    ["Utiliser des modèles déjà présents", "Use models that are already present"],
    ["Verbatim a trouvé des modèles d'IA déjà présents sur cet ordinateur (par exemple ceux d'Ollama). Cochez ceux à utiliser : ils sont",
      "Verbatim found AI models already on this computer (for example Ollama's). Tick the ones to use: they are"],
    ["copiés", "copied"], ["dans le dossier de Verbatim, les originaux ne sont ni déplacés ni modifiés.", "into Verbatim's folder; the originals are neither moved nor modified."],
    ["Dossier de destination :", "Destination folder:"], ["Choisir un dossier…", "Choose a folder…"], ["Choisir un fichier…", "Choose a file…"],
    ["Modèles personnalisés installés", "Installed custom models"],
    ["Ollama", "Ollama"], ["Si Ollama est installé, Verbatim peut l'utiliser pour rédiger avec n'importe lequel de vos modèles.", "If Ollama is installed, Verbatim can use it to write with any of your models."],
    ["Mises à jour", "Updates"],
    ["Verbatim cherche au démarrage la dernière version publiée sur GitHub. L'installation ne se lance jamais pendant un enregistrement, une transcription ou une rédaction.",
      "Verbatim checks at startup for the latest version published on GitHub. Installation never starts during a recording, a transcription or report writing."],
    ["Installer automatiquement les nouvelles versions", "Install new versions automatically"], ["Rechercher maintenant", "Check now"],
    ["API distante (facultatif)", "Remote API (optional)"], ["Utilisée si aucun modèle local n'est sélectionné.", "Used if no local model is selected."],
    ["URL de l'API", "API URL"], ["Clé d'API", "API key"], ["Réglages enregistrés", "Settings saved"],
    ["Langue de l'interface", "Interface language"], ["Automatique", "Automatic"],
    ["Calcul de la transcription", "Transcription computation"],
    ["Processeur, compatible partout", "Processor, works everywhere"], ["CPU uniquement", "CPU only"],
    ["GPU si disponible, sinon CPU", "GPU if available, otherwise CPU"],
    ["Moteurs et emplacements", "Engines and locations"], ["(installé avec le premier modèle)", "(installed with the first model)"],
    ["Aucune IA locale", "No local AI"], ["Utiliser l'API distante si elle est configurée, sinon un résumé simplifié", "Use the remote API if configured, otherwise a simplified summary"],
    ["Emplacement :", "Location:"], ["Installé :", "Installed:"], ["À installer :", "To install:"], ["Sera installé dans :", "Will be installed in:"], ["CPU :", "CPU:"], ["ROCm :", "ROCm:"], ["CUDA :", "CUDA:"],
    ["Le modèle n'est chargé en mémoire (4 à 24 Go selon le modèle) que pendant la rédaction.", "The model is only loaded into memory (4 to 24 GB depending on the model) while writing."],
    ["Processeur, compatible partout (beaucoup plus lent)", "Processor, works everywhere (much slower)"],
    ["Aucun moteur GPU installé.", "No GPU engine installed."],
    ["Moteurs et réglages avancés", "Engines and advanced settings"], ["Forcer Vulkan", "Force Vulkan"], ["(AMD, NVIDIA, Intel)", "(AMD, NVIDIA, Intel)"],
    ["Forcer CUDA", "Force CUDA"], ["(NVIDIA ; le plus rapide sur carte NVIDIA)", "(NVIDIA; fastest on NVIDIA cards)"],
    ["Forcer ROCm", "Force ROCm"], ["(AMD RDNA3 ; lecture des longs textes plus rapide)", "(AMD RDNA3; faster reading of long texts)"],
    ["Runtime ROCm :", "ROCm runtime:"], ["Runtime ROCm introuvable.", "ROCm runtime not found."],
    ["Choisir le dossier du runtime ROCm…", "Choose the ROCm runtime folder…"], ["Détection automatique", "Automatic detection"], ["Runtime ROCm trouvé", "ROCm runtime found"],
    ["Recherche d'Ollama…", "Looking for Ollama…"], ["Ollama n'est pas installé (ollama.com).", "Ollama is not installed (ollama.com)."],
    ["Démarrer Ollama", "Start Ollama"], ["Démarrage d'Ollama…", "Starting Ollama…"],
    ["Ollama fonctionne, mais aucun modèle n'est téléchargé. Dans un terminal :", "Ollama is running, but no model is downloaded. In a terminal:"],
    ["Très volumineux : sans assez de mémoire (RAM + carte graphique), la rédaction sera extrêmement lente.", "Very large: without enough memory (RAM + graphics card), writing will be extremely slow."],
    ["Gros modèle : chargement long, rédaction lente sans carte graphique.", "Large model: slow to load, slow writing without a graphics card."],
    ["Le moteur d'IA locale n'est pas installé.", "The local AI engine is not installed."], ["Télécharger le moteur (~20 Mo)", "Download the engine (~20 MB)"],
    ["Copier la sélection dans Verbatim", "Copy the selection into Verbatim"],
    ["Version", "Version"], ["Mettre à jour", "Update"], ["Plus tard", "Later"],
    ["Recherche…", "Searching…"], ["Démarrage…", "Starting…"],
    ["Aucun modèle existant à copier", "No existing model to copy"],
    ["Aucun modèle existant à copier à cet emplacement.", "No existing model to copy in this location."], ["Aucun modèle existant à copier.", "No existing model to copy."],
    ["Moteur Whisper non installé.", "Whisper engine not installed."], ["Installez d'abord le moteur Whisper (Réglages).", "Install the Whisper engine first (Settings)."],
    // Modèles (côté moteur)
    ["Très rapide, précision limitée", "Very fast, limited accuracy"], ["Rapide, précision correcte", "Fast, decent accuracy"],
    ["Bon équilibre vitesse / précision", "Good balance of speed and accuracy"], ["Medium (compressé)", "Medium (compressed)"],
    ["Précis, ~3× plus lent que Small", "Accurate, ~3× slower than Small"], ["Large v3 Turbo (compressé)", "Large v3 Turbo (compressed)"],
    ["Très précis, recommandé pour le français", "Very accurate, recommended for French"], ["Large v3 (compressé)", "Large v3 (compressed)"],
    ["Précision maximale, le plus lent", "Maximum accuracy, the slowest"],
    ["Qwen 3.5 — 4B (léger)", "Qwen 3.5 — 4B (light)"], ["Très économe : ~4 Go de RAM, rapide même sans carte graphique, compte-rendu correct", "Very frugal: ~4 GB of RAM, fast even without a graphics card, decent report"],
    ["Qwen 3.5 — 9B (équilibré, recommandé)", "Qwen 3.5 — 9B (balanced, recommended)"], ["Bon compromis qualité / mémoire : ~8 Go de RAM, repère bien décisions et actions", "Good quality / memory trade-off: ~8 GB of RAM, picks out decisions and actions well"],
    ["Qwen 3.8 — 27B allégé (IQ3)", "Qwen 3.8 — 27B lightened (IQ3)"], ["Qualité supérieure, ~14 Go de RAM ; lent sans carte graphique", "Higher quality, ~14 GB of RAM; slow without a graphics card"],
    ["Meilleure qualité ; ~24 Go de RAM conseillés, lent sans carte graphique", "Best quality; ~24 GB of RAM recommended, slow without a graphics card"],
    // Erreurs courantes
    ["Réunion introuvable", "Meeting not found"], ["Aucun compte-rendu généré.", "No report generated."], ["Il faut au moins un type de réunion.", "At least one meeting type is required."],
    ["Génération interrompue (application fermée). Relancez-la.", "Generation interrupted (application closed). Run it again."],
    ["Rien à copier.", "Nothing to copy."], ["Un enregistrement est déjà en cours.", "A recording is already in progress."], ["Aucun micro détecté.", "No microphone detected."],
    ["Aucune sortie audio détectée.", "No audio output detected."], ["Délai dépassé à l'ouverture de l'appareil audio.", "Timed out opening the audio device."],
    ["Mise à jour refusée : source ou somme de contrôle invalide.", "Update refused: invalid source or checksum."],
    ["Mise à jour annulée : somme de contrôle invalide (fichier corrompu ou modifié).", "Update cancelled: invalid checksum (file corrupted or modified)."],
    ["Aucune carte NVIDIA (pilote CUDA) détectée : le build GPU officiel de whisper.cpp ne fonctionne qu'avec NVIDIA.", "No NVIDIA card (CUDA driver) detected: the official whisper.cpp GPU build only works with NVIDIA."],
    ["Modèle inconnu.", "Unknown model."], ["Un téléchargement est déjà en cours.", "A download is already in progress."],
    ["Fréquence d'échantillonnage inconnue.", "Unknown sample rate."], ["Aucune parole détectée.", "No speech detected."],
    ["Moteur d'IA locale non installé (Réglages).", "Local AI engine not installed (Settings)."], ["Délai dépassé au chargement de l'IA locale.", "Timed out loading the local AI."],
    ["Réponse vide du modèle.", "Empty response from the model."], ["Réponse du modèle non exploitable (JSON invalide).", "Unusable model response (invalid JSON)."],
    ["Ollama ne répond pas après son démarrage.", "Ollama is not responding after starting."], ["Réponse vide d'Ollama.", "Empty response from Ollama."],
    ["Réponse d'Ollama non exploitable (JSON invalide).", "Unusable Ollama response (invalid JSON)."], ["Fichier non géré par Verbatim.", "File not supported by Verbatim."],
    ["La transcription n'est pas encore terminée.", "The transcription is not finished yet."], ["Un compte-rendu est déjà en cours de génération.", "A report is already being generated."],
    ["Ollama a échoué", "Ollama failed"],
    ["Préparer la réunion", "Prepare the meeting"], ["Quels sont les points bloquants ?", "What are the blocking points?"], ["Quelles décisions sont attendues ?", "What decisions are expected?"],
    // Langue du compte-rendu, traduction, direct (0.3)
    ["Langue du compte-rendu", "Report language"], ["Langue de rédaction du compte-rendu", "Language the report is written in"],
    ["Automatique", "Automatic"], ["Compte-rendu en français", "Report in French"], ["Compte-rendu en anglais", "Report in English"],
    ["Compte-rendu en espagnol", "Report in Spanish"], ["Compte-rendu en allemand", "Report in German"], ["Compte-rendu en italien", "Report in Italian"],
    ["Compte-rendu en portugais", "Report in Portuguese"], ["Compte-rendu en néerlandais", "Report in Dutch"],
    ["Traduction", "Translation"], ["Calcul de la traduction", "Translation compute"], ["Processeur uniquement", "Processor only"],
    ["Carte graphique si possible (CUDA, ROCm ou Vulkan), sinon processeur", "Graphics card if possible (CUDA, ROCm or Vulkan), otherwise processor"],
    ["Plus lent, mais n'utilise pas la mémoire vidéo : utile si le GPU est occupé par Whisper", "Slower, but uses no video memory: useful if the GPU is busy with Whisper"],
    ["Dossier des modèles de traduction :", "Translation models folder:"], ["Calcul utilisé actuellement :", "Compute currently used:"],
    ["Portugais", "Portuguese"], ["Néerlandais", "Dutch"], ["Polonais", "Polish"], ["Russe", "Russian"], ["Japonais", "Japanese"], ["Chinois", "Chinese"], ["Arabe", "Arabic"], ["Turc", "Turkish"], ["Coréen", "Korean"],
    ["Le compte-rendu peut être rédigé dans une autre langue que celle de la réunion. Vous pouvez aussi la choisir réunion par réunion.", "The report can be written in a language other than the meeting's. You can also choose it meeting by meeting."],
    ["Modèle de traduction", "Translation model"],
    ["Sert à traduire les transcriptions et, en direct, ce qui se dit pendant l'enregistrement. Tout reste sur cet ordinateur.", "Used to translate transcripts and, live, what is said during recording. Everything stays on this computer."],
    ["Même modèle que le compte-rendu", "Same model as the report"], ["Pratique, mais un modèle de traduction dédié donne de meilleurs résultats", "Convenient, but a dedicated translation model gives better results"],
    ["Modèle de traduction de Google (55 langues) : ~3 Go de RAM, rapide, utilisable en direct", "Google's translation model (55 languages): ~3 GB of RAM, fast, usable live"],
    ["Meilleure traduction, ~9 Go de RAM ; plus lent, surtout sans carte graphique", "Better translation, ~9 GB of RAM; slower, especially without a graphics card"],
    ["Traduire la transcription", "Translate the transcript"], ["Texte d'origine", "Original text"], ["Afficher", "Show"], ["Traduire en…", "Translate into…"],
    ["Supprimer la traduction", "Delete translation"], ["Supprimer cette traduction", "Delete this translation"], ["Supprimer cette traduction ?", "Delete this translation?"],
    ["La traduction a échoué.", "The translation failed."],
    ["Transcription en direct", "Live transcription"], ["Sans traduction", "No translation"], ["Traduire en direct", "Translate live"],
    ["Affiche le texte au fil de l'enregistrement ; la transcription complète est refaite à l'arrêt", "Shows the text as you record; the full transcription is redone when you stop"],
    ["Chargement de Whisper…", "Loading Whisper…"], ["texte provisoire : la transcription complète est refaite à l'arrêt", "provisional text: the full transcription is redone when you stop"],
    ["Langue de traduction inconnue.", "Unknown translation language."], ["Une traduction est déjà en cours.", "A translation is already in progress."],
    ["La transcription est vide.", "The transcript is empty."], ["La transcription est déjà dans cette langue.", "The transcript is already in this language."],
    ["Traduction interrompue (application fermée). Relancez-la.", "Translation interrupted (application closed). Run it again."],
    ["Cette traduction n'existe pas encore.", "This translation does not exist yet."],
    ["Aucun modèle d'IA disponible : installez un modèle dans Réglages › Compte-rendu.", "No AI model available: install a model in Settings › Report."],
    ["Le fichier du modèle est introuvable.", "The model file could not be found."],
    ["La transcription en direct nécessite whisper-server, absent de ce moteur Whisper. Réinstallez le moteur de transcription (Réglages › Transcription).", "Live transcription needs whisper-server, which is missing from this Whisper engine. Reinstall the transcription engine (Settings › Transcription)."],
    ["Aucun modèle Whisper installé : installez au moins le modèle « Base » (Réglages › Transcription).", "No Whisper model installed: install at least the “Base” model (Settings › Transcription)."],
    ["Délai dépassé au chargement de Whisper.", "Timed out while loading Whisper."],
  ].map(([k, v]) => [norm(k), v]));

  // ── Motifs : [expression, fonction(correspondance, t) → texte anglais] ──
  const P = [];
  const add = (re, fn) => P.push([re, fn]);
  const T = (s) => translate(s);
  add(/^(Moteur|Détection de parole|Modèle|Moteur IA|Modèle IA|Copie|Moteur GPU|Téléchargement) : (\d+ % \(.*\))$/, (m) => `${T(m[1])}: ${m[2]}`);
  add(/^Whisper s'exécute en local, l'audio ne quitte jamais cet ordinateur\. Modèle « (.*) » : (.+?)(?: — déjà (\d+) % téléchargés)?\. Téléchargé une seule fois, puis conservé\.$/,
    (m) => `Whisper runs locally; audio never leaves this computer. Model “${T(m[1])}”: ${m[2]}${m[3] ? ` — already ${m[3]}% downloaded` : ""}. Downloaded once, then kept.`);
  add(/^Par défaut — (.*)$/, (m) => `Default — ${m[1]}`);
  add(/^(\d+) fichier\(s\) importé\(s\)$/, (m) => `${m[1]} file(s) imported`);
  add(/^(.*?) · (Audio · transcription Whisper locale|Transcription importée)(?: · (.*))?$/, (m) => `${m[1]} · ${T(m[2])}${m[3] ? " · " + m[3] : ""}`);
  add(/^(.*) · local$/, (m) => `${T(m[1])} · local`);
  add(/^(.+) · (\d+) %$/, (m) => `${T(m[1])} · ${m[2]}%`);
  add(/^Interrompu (\d+) %$/, (m) => `Interrupted ${m[1]}%`);
  add(/^Version (.+) disponible$/, (m) => `Version ${m[1]} available`);
  add(/^Version (.+) disponible\.$/, (m) => `Version ${m[1]} available.`);
  add(/^Vous avez la dernière version \(v(.+)\)\.$/, (m) => `You have the latest version (v${m[1]}).`);
  add(/^Installation de la version (.+) : l'application va redémarrer…$/, (m) => `Installing version ${m[1]}: the application will restart…`);
  add(/^Téléchargement interrompu : (.*) — vous pourrez le reprendre\.$/, (m) => `Download interrupted: ${T(m[1])} — you can resume it later.`);
  add(/^(\d+) points?(?: · (\d+) min)? · (\d+) questions?$/, (m) => `${m[1]} item${m[1] == 1 ? "" : "s"}${m[2] ? " · " + m[2] + " min" : ""} · ${m[3]} question${m[3] == 1 ? "" : "s"}`);
  add(/^Durée prévue : (\d+) min$/, (m) => `Planned duration: ${m[1]} min`);
  add(/^Analyse de la partie (\d+)\/(\d+)…$/, (m) => `Analyzing part ${m[1]}/${m[2]}…`);
  add(/^Aucun GPU utilisable(?: \((.*)\))? : il faut une carte NVIDIA \(CUDA\) ou AMD RDNA3 \(ROCm\)\.$/, (m) => `No usable GPU${m[1] ? " (" + m[1] + ")" : ""}: an NVIDIA card (CUDA) or AMD RDNA3 (ROCm) is required.`);
  add(/^GPU actif · (.*)$/, (m) => `GPU active · ${m[1]}`);
  add(/^CPU \(GPU indisponible\)$/, () => "CPU (GPU unavailable)");
  add(/^CPU \(moteur GPU non installé\)$/, () => "CPU (GPU engine not installed)");
  add(/^GPU \((.*)\), repli sur le CPU en cas de problème$/, (m) => `GPU (${m[1]}), falls back to the CPU in case of problems`);
  add(/^GPU \((CUDA|ROCm), sinon Vulkan\), repli sur le CPU(?: · (.*))?$/, (m) => `GPU (${m[1]}, otherwise Vulkan), falls back to the CPU${m[2] ? " · " + m[2] : ""}`);
  add(/^Télécharger \((.+)\)$/, (m) => `Download (${m[1]})`);
  add(/^Télécharger le moteur (CUDA|Vulkan) \((.+)\)$/, (m) => `Download the ${m[1]} engine (${m[2]})`);
  add(/^Ollama est installé \((.*)\) mais n'est pas démarré\.$/, (m) => `Ollama is installed (${m[1]}) but is not running.`);
  add(/^Moteur d'IA locale \(llama\.cpp\) : (installé|non installé \(téléchargé avec le premier modèle\)) ·$/,
    (m) => `Local AI engine (llama.cpp): ${m[1] === "installé" ? "installed" : "not installed (downloaded with the first model)"} ·`);
  add(/^Trop volumineux pour la mémoire vidéo de votre carte \((\d+) Go\) : environ ([\d.]+) Go nécessaires\. Une partie du modèle tournera sur le processeur : rédaction très lente\.$/,
    (m) => `Too large for your card's video memory (${m[1]} GB): about ${m[2]} GB needed. Part of the model will run on the processor: writing will be very slow.`);
  add(/^Réunion « (.*) » d'environ (\d+) min\. (.*)$/s, (m) => `Meeting “${m[1]}” of about ${m[2]} min. ${m[3]}`);
  add(/^(.+) \((\d+) mentions\)$/, (m) => `${m[1]} (${m[2]} mentions)`);
  add(/^(L'IA locale|Ollama|L'API) a échoué \((.*)\)\. Résumé simplifié affiché\.$/s,
    (m) => `${{ "L'IA locale": "The local AI", "Ollama": "Ollama", "L'API": "The API" }[m[1]]} failed (${T(m[2])}). Simplified summary shown.`);
  add(/^(.*) — le moteur d'IA a été arrêté, souvent par l'antivirus qui met en quarantaine le runtime \(Vulkan, CUDA, ROCm\)\. Restaurez-le dans Sécurité Windows › Historique de protection et ajoutez le dossier « ia » aux exclusions, ou choisissez « CPU uniquement » dans Réglages › Compte-rendu$/s,
    (m) => `${T(m[1])} — the AI engine was stopped, often by the antivirus quarantining the runtime (Vulkan, CUDA, ROCm). Restore it in Windows Security › Protection history and add the “ia” folder to the exclusions, or choose “CPU only” in Settings › Report`);
  add(/^(.*) \(essai sur le processeur : (.*)\)$/s, (m) => `${T(m[1])} (retry on the processor: ${T(m[2])})`);
  add(/^L'IA locale s'est arrêtée au démarrage \((.*)\)\. Mémoire insuffisante \?$/, (m) => `The local AI stopped at startup (${m[1]}). Not enough memory?`);
  add(/^Whisper a échoué : (.*)$/s, (m) => `Whisper failed: ${m[1]}`);
  add(/^Appel du modèle impossible : (.*)$/s, (m) => `Model call failed: ${m[1]}`);
  add(/^Démarrage d'Ollama impossible : (.*)$/s, (m) => `Could not start Ollama: ${m[1]}`);
  add(/^Lancement de (l'IA locale|whisper) impossible : (.*)$/s, (m) => `Could not start ${m[1] === "whisper" ? "Whisper" : "the local AI"}: ${m[2]}`);
  add(/^Téléchargement impossible : (.*)$/s, (m) => `Download failed: ${m[1]}`);
  add(/^Téléchargement incomplet \((.*) octets\) — relancez pour reprendre\.$/, (m) => `Incomplete download (${m[1]} bytes) — run it again to resume.`);
  add(/^Connexion interrompue : (.*)$/s, (m) => `Connection interrupted: ${m[1]}`);
  add(/^Écriture impossible(?: \(disque plein \?\))? : (.*)$/s, (m) => `Write failed${/disque plein/.test(m[0]) ? " (disk full?)" : ""}: ${m[1]}`);
  add(/^Création du fichier d'enregistrement impossible : (.*)$/s, (m) => `Could not create the recording file: ${m[1]}`);
  add(/^(.*) \(vérifiez Paramètres Windows › Confidentialité › Microphone › applications de bureau\)$/s, (m) => `${m[1]} (check Windows Settings › Privacy › Microphone › desktop apps)`);
  add(/^Recherche de mise à jour impossible : (.*)$/s, (m) => `Update check failed: ${m[1]}`);
  add(/^Accès refusé à « (.*) » \((.*)\) : votre antivirus \(ou Windows Defender\) bloque ou verrouille ce fichier\. Dans Sécurité Windows › Protection contre les virus › Historique de protection, restaurez-le, ajoutez le dossier « (.*) » aux exclusions, puis relancez le téléchargement\.$/s,
    (m) => `Access denied to “${m[1]}” (${m[2]}): your antivirus (or Windows Defender) is blocking or locking this file. In Windows Security › Virus & threat protection › Protection history, restore it, add the folder “${m[3]}” to the exclusions, then restart the download.`);
  add(/^Votre antivirus a supprimé ou mis en quarantaine : (.*)\. Dans Sécurité Windows › Protection contre les virus › Historique de protection, restaurez ces fichiers, ajoutez le dossier « (.*) » aux exclusions, puis relancez le téléchargement\.$/s,
    (m) => `Your antivirus deleted or quarantined: ${m[1]}. In Windows Security › Virus & threat protection › Protection history, restore these files, add the folder “${m[2]}” to the exclusions, then restart the download.`);
  add(/^Préparer la réunion « (.*) »\.$/, (m) => `Prepare the meeting “${m[1]}”.`);
  add(/^Enregistrement récupéré \((.*)\)$/, (m) => `Recovered recording (${m[1]})`);
  add(/^Modèle « (.*) » introuvable$/, (m) => `Model “${m[1]}” not found`);
  add(/^Traduction en (.*)…$/, (m) => `Translating into ${T(m[1])}…`);
  add(/^(.*) \(refaire\)$/, (m) => `${T(m[1])} (redo)`);
  add(/^Transcription \((.*)\)$/, (m) => `Transcription (${T(m[1])})`);
  add(/^Traduire en (.*)$/, (m) => `Translate into ${T(m[1])}`);
  add(/^Chargement de la traduction : (.*)$/s, (m) => `Loading the translation: ${T(m[1])}`);
  add(/^Traduction en direct indisponible : (.*)$/s, (m) => `Live translation unavailable: ${T(m[1])}`);
  add(/^Lancement de whisper-server impossible : (.*)$/s, (m) => `Could not start whisper-server: ${m[1]}`);
  add(/^whisper-server s'est arrêté au démarrage \((.*)\)\.$/, (m) => `whisper-server stopped at startup (${m[1]}).`);
  add(/^Whisper en direct : (.*)$/s, (m) => `Live Whisper: ${m[1]}`);

  // ── Langue ──
  let pref = "auto", installLang = "";
  try { pref = localStorage.getItem("lang") || "auto"; } catch (_) {}
  const sys = () => ((navigator.language || "fr").toLowerCase().startsWith("fr") ? "fr" : "en");
  const resolve = () => (pref === "fr" || pref === "en" ? pref : installLang || sys());
  let lang = resolve();

  const units = (s) => s.replace(/(\d)\s?Go\b/g, "$1 GB").replace(/(\d)\s?Mo\b/g, "$1 MB").replace(/\bGo de RAM\b/g, "GB of RAM");
  function translate(src) {
    if (lang === "fr" || !src) return src;
    const m0 = src.match(/^(\s*)([\s\S]*?)(\s*)$/);
    const body = norm(m0[2]);
    if (!body || !/[A-Za-zÀ-ÿ]/.test(body)) return src;
    let out = EN[body];
    if (out === undefined) {
      for (const [re, fn] of P) { const m = body.match(re); if (m) { out = fn(m); break; } }
    }
    if (out === undefined && body.endsWith("…") && EN[norm(body.slice(0, -1))] !== undefined) out = EN[norm(body.slice(0, -1))] + "…";
    if (out === undefined && body.endsWith(".") && EN[norm(body.slice(0, -1))] !== undefined) out = EN[norm(body.slice(0, -1))] + ".";
    if (out === undefined && body.includes(" · ") || out === undefined && /(^· | ·$)/.test(body)) {
      const parts = body.split(" · "), tr = parts.map((x) => { const y = norm(x.replace(/^· |·$/g, "")); const z = EN[y] ?? (P.map(([re, fn]) => { const m = y.match(re); return m ? fn(m) : undefined; }).find((v) => v !== undefined)); return z === undefined ? x : x.replace(y, z); });
      if (tr.some((x, i) => x !== parts[i])) out = tr.join(" · ");
    }
    if (out === undefined) {
      // texte non reconnu (donnée de l'utilisateur, nom de fichier…) : laissé tel quel, sauf les tailles « 2.70 Go »
      return /^(· )?[\d.,]+ (Go|Mo)( · .*)?$/.test(body) ? m0[1] + units(body) + m0[3] : src;
    }
    return m0[1] + units(out) + m0[3];
  }

  // ── Application au document ──
  const SKIP = "textarea, code, script, style, [data-nt], .seg, .t, .btitle, #ttl, .tag";
  const ATTRS = ["placeholder", "title", "aria-label", "label"];
  const nodeRec = new WeakMap(), attrRec = new WeakMap();
  const skip = (el) => !el || (el.closest && el.closest(SKIP));
  function doText(n) {
    if (skip(n.parentElement)) return;
    const rec = nodeRec.get(n), cur = n.nodeValue;
    const src = rec && cur === rec.out ? rec.src : cur;
    const out = translate(src);
    nodeRec.set(n, { src, out });
    if (out !== cur) n.nodeValue = out;
  }
  function doAttrs(el) {
    if (el.closest && el.closest("[data-nt], script, style")) return;
    let m = attrRec.get(el); if (!m) attrRec.set(el, (m = new Map()));
    for (const a of ATTRS) {
      if (!el.hasAttribute(a)) continue;
      const cur = el.getAttribute(a), rec = m.get(a);
      const src = rec && cur === rec.out ? rec.src : cur;
      const out = translate(src);
      m.set(a, { src, out });
      if (out !== cur) el.setAttribute(a, out);
    }
  }
  function walk(root) {
    if (root.nodeType === 3) return doText(root);
    if (root.nodeType !== 1) return;
    doAttrs(root);
    const tw = document.createTreeWalker(root, NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT);
    for (let n = tw.nextNode(); n; n = tw.nextNode()) n.nodeType === 3 ? doText(n) : doAttrs(n);
  }
  let busy = false;
  const obs = new MutationObserver((list) => {
    if (busy) return;
    busy = true;
    try {
      for (const r of list) {
        if (r.type === "characterData") doText(r.target);
        else if (r.type === "attributes") doAttrs(r.target);
        else r.addedNodes.forEach(walk);
      }
    } finally { busy = false; obs.takeRecords(); }
  });
  function start() {
    document.documentElement.lang = lang;
    walk(document.body);
    obs.observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: ATTRS });
  }

  window.I18N = {
    get lang() { return lang; },
    get pref() { return pref; },
    t: translate,
    locale: () => (lang === "fr" ? "fr-FR" : (navigator.language || "").toLowerCase().startsWith("en") ? navigator.language : "en-US"),
    set(v) {
      pref = v; try { localStorage.setItem("lang", v); } catch (_) {}
      lang = resolve(); document.documentElement.lang = lang;
      busy = true; walk(document.body); busy = false; obs.takeRecords();
    },
    // Langue choisie dans l'installateur (utilisée tant que l'utilisateur n'a rien choisi).
    setInstall(v) { if (v !== "fr" && v !== "en") return; installLang = v; if (pref === "auto") this.set("auto"); },
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start); else start();
})();
