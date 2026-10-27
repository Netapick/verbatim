# Codes d'installation et messages d'erreur — Verbatim

Français · [English](CODES.en.md)

Mis à jour pour la version 0.4.1. L'application écrit le problème en toutes lettres ; quand c'est Windows qui refuse ou échoue, elle ajoute son code entre parenthèses (« os error 5 »). L'installateur a ses propres codes, visibles dans son journal.

## Par où commencer

| Ce que vous voyez | Aller à |
|---|---|
| « Accès refusé », moteur « arrêté », fichier « absent de l'archive », `os error 5`, `32`, `1224`, `10054` | [Antivirus](#antivirus-la-cause-la-plus-fréquente) |
| L'installation a échoué ou un composant manque | [L'installateur](#1-linstallateur) |
| Un message dans l'application | [Messages de l'application](#2-messages-de-lapplication) |
| Un `(os error N)` à la fin d'un message | [Erreurs du système](#erreurs-du-système-os-error-n) |

**Sommaire** · [Antivirus](#antivirus-la-cause-la-plus-fréquente) · [1. L'installateur](#1-linstallateur) · [2. Messages de l'application](#2-messages-de-lapplication) · [Erreurs du système](#erreurs-du-système-os-error-n)

---

## Antivirus, la cause la plus fréquente

Les moteurs d'IA (whisper.cpp, llama.cpp) ne sont pas signés : certains antivirus les suppriment ou les bloquent. Si un message parle d'« accès refusé », d'un moteur « arrêté » ou d'un fichier « absent de l'archive » :

1. Ouvrir **Sécurité Windows › Protection contre les virus › Historique de protection**.
2. Restaurer les fichiers de Verbatim.
3. Ajouter le dossier `ia` (à côté de `verbatim.exe`) aux **exclusions**.
4. En dernier recours, choisir **« CPU uniquement »** dans Réglages › Compte-rendu.

---

## 1. L'installateur

> [!TIP]
> Premier fichier à demander en cas de problème d'installation : **`redist-install.log`**, à la racine du dossier d'installation. Les lignes `[hook]` viennent de l'installateur, les autres du script de téléchargement.

### Code de détection de la carte graphique

Le script `detect-gpu.ps1` rend un nombre dont chaque bit désigne un type de carte. Le journal indique `[hook] détection GPU, code : N`.

| Bit | Carte détectée | Composant installé |
|:-:|---|---|
| **1** | NVIDIA (toute série) | Whisper CUDA (685 Mo), IA de rédaction CUDA (254 Mo) et bibliothèques CUDA (373 Mo) |
| **2** | AMD Radeon RX 7000 ou PRO W7000 (RDNA 3) | Pack ROCm (Whisper et IA de rédaction) |
| **4** | Autre carte AMD (avant la série RX 7000), ou Intel | Whisper Vulkan (17 Mo) |

Les valeurs s'additionnent quand plusieurs cartes sont présentes :

| `0` | `1` | `2` | `3` | `4` | `5` | `6` | `7` |
|---|---|---|---|---|---|---|---|
| Aucune carte reconnue (tout sur le processeur) | NVIDIA | AMD RX 7000 | NVIDIA + AMD RX 7000 | AMD ancienne ou Intel | NVIDIA + Intel (ou AMD ancienne) | AMD RX 7000 + Intel (typiquement un processeur avec carte graphique intégrée) | Les trois |

Pour la valeur 4, les cartes « Basic », « Remote » et « Virtual » (machines virtuelles, bureau à distance) sont ignorées. Un ordinateur sans GPU reconnu reste pleinement utilisable : la transcription et la rédaction passent par le processeur.

### Téléchargement d'un composant

Chaque composant est téléchargé par `dl-pack.ps1`. Le journal indique `[hook] code retour : N`.

- **`0`** : téléchargé, vérifié (somme de contrôle SHA-256) et installé.
- **`1`** : échec. La ligne `ERREUR :` qui précède donne la cause.

| Message sur la ligne `ERREUR :` | Cause et solution |
|---|---|
| Échec du téléchargement (code N) | `curl` a échoué (codes dans le tableau ci-dessous).<br>**→** Vérifier la connexion Internet, puis relancer l'installateur. |
| Somme de contrôle invalide : le fichier est corrompu ou a été modifié | Le fichier reçu n'est pas celui attendu.<br>**→** Relancer l'installateur ; si cela persiste, un proxy ou un antivirus modifie le téléchargement. |
| Archive invalide | Le zip contient un chemin qui sort du dossier d'installation.<br>**→** Ne pas poursuivre : télécharger à nouveau l'installateur depuis la page officielle. |

<details>
<summary><b>Codes de <code>curl</code></b> (le <code>N</code> de « Échec du téléchargement »)</summary>

| N | Signification |
|:-:|---|
| 6 | Nom du serveur introuvable (DNS, pas de connexion) |
| 7 | Connexion impossible (pare-feu, proxy) |
| 22 | Le serveur a répondu par une erreur (404 : le fichier n'existe pas à cette adresse) |
| 28 | Délai dépassé |
| 35 | Échec de la connexion sécurisée (SSL/TLS) |
| 56 | Connexion coupée pendant le téléchargement |
| 60 | Certificat du serveur refusé (heure de l'ordinateur fausse, proxy qui inspecte le trafic) |

</details>

> [!NOTE]
> Un échec ne bloque pas l'installation : Verbatim fonctionne, sans le composant concerné. Il suffit de relancer l'installateur ; les composants déjà présents ne sont pas téléchargés à nouveau.

<details>
<summary><b>Code de sortie et options de la ligne de commande</b></summary>

**Code de sortie de l'installateur** (convention de NSIS) : `0` installation réussie · `1` annulée par l'utilisateur · `2` interrompue par le script.

**Options de la ligne de commande**, utilisées par la mise à jour intégrée et utiles pour déployer sans interface :

| Option | Effet |
|---|---|
| `/S` | Silencieux : aucune fenêtre ni boîte de dialogue |
| `/P` | Passif : une barre de progression, sans question (mode utilisé par la mise à jour de l'application) |
| `/UPDATE` | Indique une mise à jour : conserve les données et les composants déjà installés |
| `/R` | Relance l'application à la fin |
| `/D=<dossier>` | Dossier d'installation (doit être la dernière option) |

</details>

**Où sont les fichiers ?** Les composants téléchargés pendant l'installation sont dans `redist`. Les modèles et moteurs téléchargés ensuite par l'application sont dans `ia`.

---

## 2. Messages de l'application

Dans chaque tableau, la deuxième colonne donne la cause puis, après **→**, ce qu'il faut faire.

<details>
<summary><b>Installation et téléchargements</b> · 9 messages</summary>

| Message | Cause → que faire |
|---|---|
| Moteur Whisper non installé. / Installez d'abord le moteur Whisper (Réglages). | Aucun moteur de transcription.<br>**→** Cliquer sur « Installer Whisper » (carte sous « Réunions » ou Réglages › Transcription). |
| Aucun modèle Whisper installé : installez au moins le modèle « Base » | Le moteur est là mais pas de modèle.<br>**→** Réglages › Transcription. |
| Un téléchargement est déjà en cours. | Un seul téléchargement à la fois.<br>**→** Attendre la fin, ou « Arrêter ». |
| Téléchargement impossible : … | Réseau, ou serveur indisponible.<br>**→** Vérifier la connexion ; le téléchargement reprend là où il s'est arrêté. |
| Téléchargement annulé. | Vous avez cliqué sur « Arrêter ».<br>**→** Rien : la partie déjà reçue est supprimée. |
| Écriture impossible (disque plein ?) | Plus de place sur le disque.<br>**→** Libérer de l'espace. |
| Accès refusé à « … » : votre antivirus (ou Windows Defender) bloque ou verrouille ce fichier | Un antivirus analyse ou met en quarantaine le fichier.<br>**→** Voir [Antivirus](#antivirus-la-cause-la-plus-fréquente). |
| Votre antivirus a supprimé ou mis en quarantaine : … | Un moteur d'IA non signé a été retiré juste après son extraction.<br>**→** Restaurer les fichiers dans Sécurité Windows et exclure le dossier `ia`. |
| … absent de l'archive | Archive de moteur incomplète.<br>**→** Réessayer ; si cela persiste, le moteur a été retiré par l'antivirus. |

</details>

<details>
<summary><b>Transcription et séparation des intervenants</b> · 10 messages</summary>

| Message | Cause → que faire |
|---|---|
| Audio vide ou illisible. / Aucune piste audio. | Fichier endommagé ou format non géré.<br>**→** Convertir le fichier (mp3, wav) et le réimporter. |
| La transcription a planté (fichier audio illisible ?) | Le moteur s'est arrêté pendant le travail.<br>**→** Réessayer, ou réimporter le fichier. |
| Aucune parole détectée. | Le détecteur de parole n'a rien trouvé.<br>**→** Vérifier le niveau du micro et le fichier. |
| La transcription est vide. | Aucun texte produit.<br>**→** Essayer un modèle plus gros. |
| Délai dépassé au chargement de Whisper. | Le moteur n'a pas démarré à temps (souvent l'antivirus ou le pilote graphique).<br>**→** Choisir « Processeur » dans Réglages › Transcription, exclure le dossier `ia` de l'antivirus. |
| La transcription en direct nécessite whisper-server, absent de ce moteur Whisper | Ancien moteur sans serveur.<br>**→** Réinstaller le moteur de transcription. |
| Whisper n'est pas encore chargé : attendez la fin du chargement avant de démarrer | Le préchargement est en cours (direct).<br>**→** Attendre quelques secondes. |
| Téléchargez d'abord les modèles de séparation des intervenants | Modèles absents.<br>**→** « Installer la séparation des intervenants ». |
| Modèles de séparation des intervenants illisibles : supprimez-les et téléchargez-les à nouveau | Fichiers endommagés.<br>**→** Réglages › Transcription, supprimer puis réinstaller. |
| La séparation des intervenants a échoué. / … a planté. | Erreur pendant le calcul.<br>**→** Relancer « Identifier les intervenants ». |

</details>

<details>
<summary><b>Enregistrement</b> · 7 messages</summary>

| Message | Cause → que faire |
|---|---|
| Aucun micro détecté. / Aucune sortie audio détectée. | Appareil introuvable.<br>**→** Brancher l'appareil, ou choisir un autre appareil dans le panneau d'enregistrement. |
| Impossible d'ouvrir l'appareil audio … (vérifiez Paramètres Windows › Confidentialité › Microphone) | Windows refuse l'accès.<br>**→** Autoriser les applications de bureau à utiliser le micro. |
| Délai dépassé à l'ouverture de l'appareil audio. | L'appareil ne répond pas.<br>**→** Le débrancher et le rebrancher. |
| Choisissez au moins une source. | Ni micro ni son du PC.<br>**→** Cocher au moins une source. |
| Un enregistrement est déjà en cours. | Un seul enregistrement à la fois.<br>**→** Terminer le premier. |
| Enregistrement trop court (moins d'une seconde). | Rien à conserver.<br>**→** Recommencer. |
| Création du fichier d'enregistrement impossible | Disque plein ou dossier inaccessible.<br>**→** Libérer de l'espace. |

> [!NOTE]
> Après un plantage ou une coupure, l'enregistrement est reconstitué au lancement suivant sous le titre « Enregistrement récupéré ».

</details>

<details>
<summary><b>Compte-rendu et traduction</b> · 10 messages</summary>

| Message | Cause → que faire |
|---|---|
| Aucun modèle d'IA disponible : installez un modèle dans Réglages › Compte-rendu | Aucune IA choisie.<br>**→** Carte « Modèle de compte-rendu non installé ». |
| Le moteur d'IA locale (llama.cpp) n'est pas installé | Moteur absent.<br>**→** Télécharger un modèle : le moteur suit. |
| Délai dépassé au chargement de l'IA locale. | Souvent un modèle trop gros pour la mémoire, ou un moteur arrêté.<br>**→** Choisir un modèle plus petit ou « CPU uniquement ». |
| … le moteur d'IA a été arrêté, souvent par l'antivirus | Runtime (Vulkan, CUDA, ROCm) mis en quarantaine.<br>**→** Restaurer dans Sécurité Windows, exclure `ia`, ou choisir « CPU uniquement ». |
| Réponse du modèle coupée : elle dépasse la longueur maximale autorisée | Le plus souvent une réunion très longue.<br>**→** Réessayer ; un modèle plus gros aide. |
| Réponse vide du modèle. | L'IA n'a rien produit.<br>**→** Réessayer. |
| Un compte-rendu est déjà en cours de génération. / Une traduction est déjà en cours. | Une seule tâche à la fois.<br>**→** Attendre. |
| La transcription n'est pas encore terminée. / La réunion n'est pas prête. | La réunion est encore en traitement.<br>**→** Attendre la fin. |
| Cette traduction n'existe pas encore. | Export d'une langue non traduite.<br>**→** Traduire d'abord. |
| Runtime ROCm introuvable dans ce dossier (attendu : amdhip64_7.dll et rocblas.dll) | Le dossier indiqué n'est pas un runtime ROCm.<br>**→** Choisir le dossier qui contient ces deux fichiers. |

</details>

<details>
<summary><b>Mise à jour</b> · 2 messages</summary>

| Message | Cause → que faire |
|---|---|
| Un enregistrement / une transcription / une rédaction / une traduction / un téléchargement est en cours. | La mise à jour est refusée tant qu'un travail tourne.<br>**→** Attendre la fin, puis relancer. |
| Erreurs de mise à jour possibles : hors ligne, signature invalide, téléchargement interrompu | Réseau, ou fichier modifié.<br>**→** « Réessayer » ; une signature invalide indique un fichier modifié : ne pas insister. |

</details>

---

## Erreurs du système, os error N

Quand Windows refuse une opération sur un fichier ou une connexion, le message de l'application se termine par `(os error N)`, précédé du texte de Windows dans la langue du système. Exemple : « Accès refusé. (os error 5) ».

| Code | Nom Windows | Cause → que faire |
|:-:|---|---|
| **2**, **3** | Fichier ou chemin introuvable | Un fichier ou un dossier a disparu (déplacé, supprimé, ou retiré par l'antivirus).<br>**→** Réinstaller le composant concerné. |
| **5** | Accès refusé | Droits insuffisants, ou un antivirus bloque le fichier.<br>**→** Voir [Antivirus](#antivirus-la-cause-la-plus-fréquente) ; ne pas installer dans un dossier protégé (`Program Files`). |
| **8**, **1455** | Mémoire insuffisante | Pas assez de mémoire ou de fichier d'échange pour charger le modèle.<br>**→** Fermer d'autres applications, choisir un modèle plus petit, ou augmenter le fichier d'échange de Windows. |
| **32**, **33** | Violation de partage ou de verrouillage | Un autre programme (souvent un antivirus qui analyse) utilise le fichier.<br>**→** Attendre quelques secondes et réessayer ; exclure le dossier `ia`. |
| **80**, **183** | Le fichier existe déjà | Un fichier du même nom est présent.<br>**→** Réessayer ; supprimer le fichier gênant. |
| **112** | Espace insuffisant sur le disque | Disque plein.<br>**→** Libérer de l'espace (les modèles pèsent plusieurs Go). |
| **123**, **206** | Nom ou chemin invalide, trop long | Dossier d'installation trop profond.<br>**→** Choisir un dossier plus court. |
| **225**, **226** | Fichier contenant un virus (supprimé) | Un antivirus a retiré le fichier.<br>**→** Voir [Antivirus](#antivirus-la-cause-la-plus-fréquente). |
| **1224** | Fichier ouvert par un autre programme | Un antivirus analyse le fichier juste après son extraction.<br>**→** Réessayer ; exclure le dossier `ia`. |
| **10053**, **10054**, **10061** | Connexion interrompue, réinitialisée, refusée | Le moteur d'IA s'est arrêté ou a été tué en cours de route.<br>**→** Voir [Antivirus](#antivirus-la-cause-la-plus-fréquente) ; sinon choisir « CPU uniquement ». |
| **10060** | Délai de connexion dépassé | Réseau ou pare-feu.<br>**→** Réessayer ; vérifier le pare-feu. |

Les codes 5, 32, 33 et 1224 déclenchent le message « Accès refusé à … : votre antivirus … bloque ou verrouille ce fichier ». Les codes 10053, 10054 et 10061 déclenchent le message « le moteur d'IA a été arrêté, souvent par l'antivirus ».
