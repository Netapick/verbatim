# Codes d'installation et messages d'erreur — Verbatim

Français · [English](CODES.en.md)

Mis à jour pour la version 0.4.0. L'application écrit le problème en toutes lettres. Quand c'est Windows qui refuse ou échoue, elle ajoute le code du système entre parenthèses (« os error 5 »). L'installateur a ses propres codes, visibles dans son journal. Cette note regroupe les trois.

## 1. L'installateur

### Journal
L'installation des composants (moteurs, accélérations GPU) écrit son déroulé dans `redist-install.log`, à la racine du dossier d'installation. Les lignes `[hook]` sont celles de l'installateur, les autres viennent du script de téléchargement. C'est le premier fichier à demander en cas de problème d'installation.

### Code de détection de la carte graphique
Le script `detect-gpu.ps1` rend un nombre dont chaque bit désigne un type de carte. Il apparaît dans le journal : `[hook] détection GPU, code : N`.

| Valeur | Carte détectée | Composant installé |
|---|---|---|
| 1 | NVIDIA (toute série) | Whisper CUDA (685 Mo), IA de rédaction CUDA (254 Mo) et bibliothèques CUDA (373 Mo) |
| 2 | AMD Radeon RX 7000 ou PRO W7000 (RDNA 3) | Pack ROCm (Whisper et IA de rédaction) |
| 4 | Autre carte AMD (avant la série RX 7000), ou Intel | Whisper Vulkan (17 Mo) |

Les valeurs s'additionnent quand plusieurs cartes sont présentes.

| Code | Signification |
|---|---|
| 0 | Aucune carte reconnue : tout se fait sur le processeur |
| 1 | NVIDIA |
| 2 | AMD RX 7000 |
| 3 | NVIDIA + AMD RX 7000 |
| 4 | AMD ancienne ou Intel |
| 5 | NVIDIA + Intel (ou AMD ancienne) |
| 6 | AMD RX 7000 + Intel (typiquement un processeur avec carte graphique intégrée) |
| 7 | Les trois |

Pour la valeur 4, les cartes « Basic », « Remote » et « Virtual » (machines virtuelles, bureau à distance) sont ignorées. Un ordinateur sans GPU reconnu reste pleinement utilisable : la transcription et la rédaction passent par le processeur.

### Code de retour d'un téléchargement
Chaque composant est téléchargé par `dl-pack.ps1`. Le journal indique `[hook] code retour : N`.

| Code | Signification |
|---|---|
| 0 | Téléchargé, vérifié (somme de contrôle SHA-256) et installé |
| 1 | Échec : la ligne `ERREUR :` qui précède donne la cause |

Causes possibles sur la ligne `ERREUR :`

| Message | Cause | Que faire |
|---|---|---|
| Échec du téléchargement (code N) | `curl` a échoué ; voir le tableau ci-dessous | Vérifier la connexion Internet, puis relancer l'installateur |
| Somme de contrôle invalide : le fichier est corrompu ou a été modifié | Le fichier reçu n'est pas celui attendu | Relancer l'installateur ; si cela persiste, un proxy ou un antivirus modifie le téléchargement |
| Archive invalide | Le zip contient un chemin qui sort du dossier d'installation | Ne pas poursuivre : télécharger à nouveau l'installateur depuis la page officielle |

Le code `N` de « Échec du téléchargement » est celui de `curl` :

| N | Signification |
|---|---|
| 6 | Nom du serveur introuvable (DNS, pas de connexion) |
| 7 | Connexion impossible (pare-feu, proxy) |
| 22 | Le serveur a répondu par une erreur (404 : le fichier n'existe pas à cette adresse) |
| 28 | Délai dépassé |
| 35 | Échec de la connexion sécurisée (SSL/TLS) |
| 56 | Connexion coupée pendant le téléchargement |
| 60 | Certificat du serveur refusé (heure de l'ordinateur fausse, proxy qui inspecte le trafic) |

Un échec ne bloque pas l'installation : Verbatim fonctionne, sans le composant concerné. Il suffit de relancer l'installateur. Les composants déjà présents ne sont pas téléchargés à nouveau.

### Code de sortie de l'installateur
Il suit la convention de NSIS : `0` installation réussie, `1` annulée par l'utilisateur, `2` interrompue par le script.

### Options de la ligne de commande
Utilisées par la mise à jour intégrée ; utiles pour déployer sans interface.

| Option | Effet |
|---|---|
| `/S` | Silencieux : aucune fenêtre ni boîte de dialogue |
| `/P` | Passif : une barre de progression, sans question (mode utilisé par la mise à jour de l'application) |
| `/UPDATE` | Indique une mise à jour : conserve les données et les composants déjà installés |
| `/R` | Relance l'application à la fin |
| `/D=<dossier>` | Dossier d'installation (doit être la dernière option) |

Les composants téléchargés pendant l'installation se trouvent dans `redist`. Les modèles et moteurs téléchargés ensuite par l'application sont dans `ia`.

## 2. Messages de l'application

### Installation et téléchargements
| Message | Cause | Que faire |
|---|---|---|
| Moteur Whisper non installé. / Installez d'abord le moteur Whisper (Réglages). | Aucun moteur de transcription | Cliquer sur « Installer Whisper » (carte sous « Réunions » ou Réglages › Transcription) |
| Aucun modèle Whisper installé : installez au moins le modèle « Base » | Le moteur est là mais pas de modèle | Réglages › Transcription |
| Un téléchargement est déjà en cours. | Un seul téléchargement à la fois | Attendre la fin, ou « Arrêter » |
| Téléchargement impossible : … | Réseau, ou serveur indisponible | Vérifier la connexion ; le téléchargement reprend là où il s'est arrêté |
| Téléchargement annulé. | Vous avez cliqué sur « Arrêter » | Aucun : la partie déjà reçue est supprimée |
| Écriture impossible (disque plein ?) | Plus de place sur le disque | Libérer de l'espace |
| Accès refusé à « … » : votre antivirus (ou Windows Defender) bloque ou verrouille ce fichier | Un antivirus analyse ou met en quarantaine le fichier | Voir « Antivirus » ci-dessous |
| Votre antivirus a supprimé ou mis en quarantaine : … | Un moteur d'IA non signé a été retiré juste après son extraction | Restaurer les fichiers dans Sécurité Windows et exclure le dossier `ia` |
| … absent de l'archive | Archive de moteur incomplète | Réessayer ; si cela persiste, le moteur a été retiré par l'antivirus |

### Transcription et séparation des intervenants
| Message | Cause | Que faire |
|---|---|---|
| Audio vide ou illisible. / Aucune piste audio. | Fichier endommagé ou format non géré | Convertir le fichier (mp3, wav) et le réimporter |
| La transcription a planté (fichier audio illisible ?) | Le moteur s'est arrêté pendant le travail | Réessayer, ou réimporter le fichier |
| Aucune parole détectée. | Le détecteur de parole n'a rien trouvé | Vérifier le niveau du micro et le fichier |
| La transcription est vide. | Aucun texte produit | Essayer un modèle plus gros |
| Délai dépassé au chargement de Whisper. | Le moteur n'a pas démarré à temps (souvent l'antivirus ou le pilote graphique) | Choisir « Processeur » dans Réglages › Transcription, exclure le dossier `ia` de l'antivirus |
| La transcription en direct nécessite whisper-server, absent de ce moteur Whisper | Ancien moteur sans serveur | Réinstaller le moteur de transcription |
| Whisper n'est pas encore chargé : attendez la fin du chargement avant de démarrer | Le préchargement est en cours (direct) | Attendre quelques secondes |
| Téléchargez d'abord les modèles de séparation des intervenants | Modèles absents | « Installer la séparation des intervenants » |
| Modèles de séparation des intervenants illisibles : supprimez-les et téléchargez-les à nouveau | Fichiers endommagés | Réglages › Transcription, supprimer puis réinstaller |
| La séparation des intervenants a échoué. / … a planté. | Erreur pendant le calcul | Relancer « Identifier les intervenants » |

### Enregistrement
| Message | Cause | Que faire |
|---|---|---|
| Aucun micro détecté. / Aucune sortie audio détectée. | Appareil introuvable | Brancher l'appareil, ou choisir un autre appareil dans le panneau d'enregistrement |
| Impossible d'ouvrir l'appareil audio … (vérifiez Paramètres Windows › Confidentialité › Microphone) | Windows refuse l'accès | Autoriser les applications de bureau à utiliser le micro |
| Délai dépassé à l'ouverture de l'appareil audio. | L'appareil ne répond pas | Le débrancher et le rebrancher |
| Choisissez au moins une source. | Ni micro ni son du PC | Cocher au moins une source |
| Un enregistrement est déjà en cours. | Un seul enregistrement à la fois | Terminer le premier |
| Enregistrement trop court (moins d'une seconde). | Rien à conserver | Recommencer |
| Création du fichier d'enregistrement impossible | Disque plein ou dossier inaccessible | Libérer de l'espace |

Après un plantage ou une coupure, l'enregistrement est reconstitué au lancement suivant sous le titre « Enregistrement récupéré ».

### Compte-rendu et traduction
| Message | Cause | Que faire |
|---|---|---|
| Aucun modèle d'IA disponible : installez un modèle dans Réglages › Compte-rendu | Aucune IA choisie | Carte « Modèle de compte-rendu non installé » |
| Le moteur d'IA locale (llama.cpp) n'est pas installé | Moteur absent | Télécharger un modèle : le moteur suit |
| Délai dépassé au chargement de l'IA locale. | Souvent un modèle trop gros pour la mémoire, ou un moteur arrêté | Choisir un modèle plus petit ou « CPU uniquement » |
| … le moteur d'IA a été arrêté, souvent par l'antivirus | Runtime (Vulkan, CUDA, ROCm) mis en quarantaine | Restaurer dans Sécurité Windows, exclure `ia`, ou choisir « CPU uniquement » |
| Réponse du modèle coupée : elle dépasse la longueur maximale autorisée | Le plus souvent une réunion très longue | Réessayer ; un modèle plus gros aide |
| Réponse vide du modèle. | L'IA n'a rien produit | Réessayer |
| Un compte-rendu est déjà en cours de génération. / Une traduction est déjà en cours. | Une seule tâche à la fois | Attendre |
| La transcription n'est pas encore terminée. / La réunion n'est pas prête. | La réunion est encore en traitement | Attendre la fin |
| Cette traduction n'existe pas encore. | Export d'une langue non traduite | Traduire d'abord |
| Runtime ROCm introuvable dans ce dossier (attendu : amdhip64_7.dll et rocblas.dll) | Le dossier indiqué n'est pas un runtime ROCm | Choisir le dossier qui contient ces deux fichiers |

### Mise à jour
| Message | Cause | Que faire |
|---|---|---|
| Un enregistrement / une transcription / une rédaction / une traduction / un téléchargement est en cours. | La mise à jour est refusée tant qu'un travail tourne | Attendre la fin, puis relancer |
| Erreurs de mise à jour possibles : hors ligne, signature invalide, téléchargement interrompu | Réseau, ou fichier modifié | « Réessayer » ; une signature invalide indique un fichier modifié : ne pas insister |

### Erreurs du système (« os error N »)
Quand Windows refuse une opération sur un fichier ou une connexion, le message de l'application se termine par `(os error N)`, précédé du texte de Windows dans la langue du système. Exemple : « Accès refusé. (os error 5) ». Les plus courants :

| Code | Nom Windows | Signification | Que faire |
|---|---|---|---|
| 2, 3 | Fichier ou chemin introuvable | Un fichier ou un dossier a disparu (déplacé, supprimé, ou retiré par l'antivirus) | Réinstaller le composant concerné |
| 5 | Accès refusé | Droits insuffisants, ou un antivirus bloque le fichier | Voir « Antivirus » ; ne pas installer dans un dossier protégé (`Program Files`) |
| 8, 1455 | Mémoire insuffisante | Pas assez de mémoire ou de fichier d'échange pour charger le modèle | Fermer d'autres applications, choisir un modèle plus petit, ou augmenter le fichier d'échange de Windows |
| 32, 33 | Violation de partage ou de verrouillage | Un autre programme (souvent un antivirus qui analyse) utilise le fichier | Attendre quelques secondes et réessayer ; exclure le dossier `ia` |
| 80, 183 | Le fichier existe déjà | Un fichier du même nom est présent | Réessayer ; supprimer le fichier gênant |
| 112 | Espace insuffisant sur le disque | Disque plein | Libérer de l'espace (les modèles pèsent plusieurs Go) |
| 123, 206 | Nom ou chemin invalide, trop long | Dossier d'installation trop profond | Choisir un dossier plus court |
| 225, 226 | Fichier contenant un virus (supprimé) | Un antivirus a retiré le fichier | Voir « Antivirus » |
| 1224 | Fichier ouvert par un autre programme | Un antivirus analyse le fichier juste après son extraction | Réessayer ; exclure le dossier `ia` |
| 10053, 10054, 10061 | Connexion interrompue, réinitialisée, refusée | Le moteur d'IA s'est arrêté ou a été tué en cours de route | Voir « Antivirus » ; sinon choisir « CPU uniquement » |
| 10060 | Délai de connexion dépassé | Réseau ou pare-feu | Réessayer ; vérifier le pare-feu |

Les codes 5, 32, 33 et 1224 déclenchent le message « Accès refusé à … : votre antivirus … bloque ou verrouille ce fichier ». Les codes 10053, 10054 et 10061 déclenchent le message « le moteur d'IA a été arrêté, souvent par l'antivirus ».

### Antivirus
Les moteurs d'IA (whisper.cpp, llama.cpp) ne sont pas signés : certains antivirus les suppriment. Si un message parle d'« accès refusé », d'un moteur « arrêté » ou d'un fichier « absent de l'archive » :

1. Ouvrir Sécurité Windows › Protection contre les virus › Historique de protection.
2. Restaurer les fichiers de Verbatim.
3. Ajouter le dossier `ia` (à côté de `verbatim.exe`) aux exclusions.
4. En dernier recours, choisir « CPU uniquement » dans Réglages › Compte-rendu.
