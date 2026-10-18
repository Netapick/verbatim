<div align="center">

<img src="src-tauri/icons/icon.png" alt="Logo Verbatim" width="110">

# Verbatim

### Vos réunions, transcrites et résumées. Chez vous, sur votre PC.

Verbatim enregistre ou importe vos réunions, les transcrit avec **Whisper**, puis rédige le compte-rendu avec une IA **Qwen** qui tourne sur votre propre ordinateur.<br>
Aucun compte, aucun abonnement, aucune télémétrie : **rien ne quitte votre machine.**

[![Release](https://img.shields.io/github/v/release/Netapick/verbatim)](https://github.com/Netapick/verbatim/releases/latest) [![Licence MIT](https://img.shields.io/github/license/Netapick/verbatim)](LICENSE) [![Téléchargements](https://img.shields.io/github/downloads/Netapick/verbatim/total)](https://github.com/Netapick/verbatim/releases) ![Windows x64](https://img.shields.io/badge/Windows-x64-0078d4) ![NVIDIA CUDA](https://img.shields.io/badge/NVIDIA-CUDA-76B900?logo=nvidia&logoColor=white) ![AMD ROCm](https://img.shields.io/badge/AMD-ROCm%20%2B%20Vulkan-ED1C24?logo=amd&logoColor=white)

**[⬇ Télécharger](https://github.com/Netapick/verbatim/releases/latest)** &nbsp;·&nbsp; [Fonctionnalités](#fonctionnalités) &nbsp;·&nbsp; [Installation](#installation) &nbsp;·&nbsp; [FAQ](#faq) &nbsp;·&nbsp; [Feuille de route](#feuille-de-route) &nbsp;·&nbsp; [Documentation](#documentation)

Français · [English](README.en.md)

<br>

<img src="docs/screenshots/03-compte-rendu.png" alt="Un compte-rendu généré par Verbatim, avec la transcription minutée" width="860">

</div>

<br>

## Pourquoi Verbatim ?

| Atout | En pratique |
|---|---|
| 🔒 **100 % local** | L'audio, les transcriptions, les comptes-rendus et les modèles restent sur votre ordinateur. Pas de cloud, pas de compte, pas d'envoi de données. |
| ⚡ **Rapide grâce à votre carte graphique** | NVIDIA (CUDA) ou AMD (ROCm, Vulkan), avec repli automatique sur le processeur quand aucune carte n'est utilisable. |
| 📝 **Du son au compte-rendu** | Résumé, points abordés, décisions, actions et prochaines étapes, à partir d'un simple enregistrement. |
| 🧰 **Pensé pour les réunions** | Préparations (ordre du jour, questions), glossaire de sigles métier, lecteur audio synchronisé avec la transcription. |

## Comment ça marche

1. **Enregistrez** une réunion (micro, son du PC ou les deux) ou **importez** un audio ou une transcription existante.
2. **Transcrivez** : Whisper convertit la parole en texte minuté, localement.
3. **Rédigez** : une IA locale produit le compte-rendu, que vous relisez, exportez ou régénérez.

## Fonctionnalités

### Enregistrer ou importer

<img src="docs/screenshots/02-enregistrement.png" alt="Panneau de nouvel enregistrement" width="720">

- Capture du **microphone**, du **son du PC** (visio, vidéo, via la boucle WASAPI) ou des deux, mixés en mono. Niveaux en direct, pause et reprise, réglage de la sensibilité.
- L'audio est écrit sur le disque au fil de l'eau : après un plantage ou une coupure, l'enregistrement est reconstitué au prochain lancement (environ 2 secondes perdues au plus).
- Import d'**audio** (mp3, m4a/aac, ogg, flac, wav…) ou d'une **transcription existante** (txt, srt, vtt, md).

### Transcrire avec Whisper

- Whisper local (whisper.cpp), avec détection de parole pour limiter les hallucinations sur les silences.
- Français, anglais, espagnol, allemand, italien, ou détection automatique de la langue.
- Du modèle le plus léger (Tiny, 75 Mo) au plus précis (Large v3), avec des versions compressées.
- Un glossaire de sigles et de termes métier est transmis au modèle pour limiter les erreurs sur les noms propres.

<img src="docs/screenshots/07-reglages-transcription.png" alt="Réglages de la transcription" width="720">

### Rédiger le compte-rendu

- Résumé, points abordés, décisions, actions et prochaines étapes, générés par une IA locale : **Qwen 3.5** (4B ou 9B), **Qwen 3.8 27B**, ou vos propres modèles GGUF. Ollama et une API compatible OpenAI restent possibles en option.
- Les réunions longues sont découpées en parties, résumées, puis fusionnées. Le glossaire et la préparation liée sont fournis au modèle.
- Sans modèle de rédaction, Verbatim produit un résumé simplifié à partir de phrases réelles de la transcription, sans rien inventer.
- Le modèle n'est chargé en mémoire que pendant la rédaction.

<img src="docs/screenshots/08-reglages-redaction.png" alt="Réglages de la rédaction" width="720">

### Préparer et retrouver

<img src="docs/screenshots/04-preparations.png" alt="Préparations de réunion" width="720">

- **Préparations** : objectif, ordre du jour chronométré et questions à poser, avec des types de réunion personnalisables.
- **Lecteur audio synchronisé** (0,75× à 2×) : un clic sur un passage de la transcription lance la lecture à cet instant.
- **Export** en texte, sous-titres (.srt, .vtt), données (.json), compte-rendu (.md) et audio d'origine.
- **Thème clair et sombre**, interface en français et en anglais.

## Matériel pris en charge

| Carte graphique | Transcription (Whisper) | Rédaction (Qwen) |
|---|---|---|
| **NVIDIA** | CUDA | CUDA, sinon Vulkan |
| **AMD Radeon RX 7000 / PRO W7000** (RDNA 3) | ROCm | ROCm, sinon Vulkan |
| Autres AMD, Intel | Processeur | Vulkan |
| Sans carte graphique | Processeur | Processeur |

Le mode **Automatique** choisit le moteur le plus rapide disponible et revient sur le processeur en cas de problème. Les modèles trop volumineux pour la mémoire vidéo de votre carte sont signalés dans les Réglages.

**Vitesse de rédaction** (RX 7800 XT, Qwen 3.5 4B Q4, en tokens par seconde) :

| Moteur | Lecture du texte | Écriture |
|---|---:|---:|
| Processeur | 94 | 11 |
| Vulkan | 2 566 | 127 |
| ROCm | 3 673 | 117 |

### Whisper sur carte AMD : un build ROCm fait pour Verbatim

whisper.cpp ne fournit pas de binaire pour utiliser une carte AMD sous Windows : seules les versions processeur et NVIDIA (CUDA) sont publiées, et la transcription accélérée sur AMD n'est pas prévue clé en main. Verbatim comble ce manque :

- **whisper.cpp est compilé avec ROCm (HIP)** pour les Radeon RX 7000 / PRO W7000 (RDNA 3), avec le script `scripts/build-whisper-rocm.ps1`.
- **Un runtime ROCm minimal** est livré avec lui, pour que la carte fonctionne sans installer le SDK d'AMD.
- **L'installateur le met en place seul** quand il détecte une carte compatible, et le mode « Automatique » le choisit, avec repli sur le processeur en cas de problème.

Testé sur une RX 7800 XT.

## Installation

1. **Téléchargez** `Verbatim_X.Y.Z_x64-setup.exe` depuis la [dernière release](https://github.com/Netapick/verbatim/releases/latest). L'installateur est léger (environ 3 Mo) et propose le français ou l'anglais.
2. **Lancez-le.** Il télécharge seul les moteurs : le pack de base (≈ 61 Mo), puis l'accélération adaptée à votre carte (ROCm ≈ 430 Mo, ou CUDA ≈ 1,3 Go). Il ne pose aucune question et ne demande pas de droits administrateur.
3. **Choisissez vos modèles** dans **Réglages** : un modèle de transcription et, si vous le souhaitez, un modèle de rédaction. Aucun modèle n'est livré avec l'application.

Les moteurs et les modèles sont rangés dans un dossier `ia` à côté de l'application. Si vous avez déjà des modèles (Ollama, fichiers `.gguf` ou `ggml-*.bin`), **Réglages › Modèles** les détecte et les **copie** dans `ia`, sans toucher aux originaux.

### Mises à jour automatiques

Au démarrage, Verbatim cherche la dernière version publiée sur GitHub, vérifie son empreinte SHA-256, l'installe, puis se relance. Cela n'arrive **jamais** pendant un enregistrement, une transcription ou une rédaction : une bannière propose alors la mise à jour. Désactivable dans **Réglages › Connexions**.

### Si votre antivirus bloque un moteur

Les moteurs tiers (whisper.cpp, llama.cpp) ne sont pas signés : certains antivirus, dont Windows Defender, peuvent les supprimer ou les bloquer (erreur « accès refusé » ou « connexion fermée »).

1. Ouvrez *Sécurité Windows › Protection contre les virus et menaces › Gérer les paramètres › Exclusions*.
2. Ajoutez le dossier d'installation de Verbatim (celui qui contient `ia`) comme exclusion.
3. Restaurez les fichiers dans *Historique de protection*, ou relancez le téléchargement depuis les Réglages.

Lancer l'application en administrateur n'aide pas.

## Confidentialité

| Ce qui reste sur votre ordinateur | Ce qui sort, et seulement si vous le demandez |
|---|---|
| Audio, transcriptions, comptes-rendus, préparations, glossaire, modèles | Téléchargement des moteurs et des modèles (GitHub, Hugging Face) |
| Toute la transcription et la rédaction | Recherche de mise à jour auprès de l'API GitHub (désactivable) |
| | Une API distante de rédaction, uniquement si vous la configurez vous-même |

Aucune donnée personnelle n'est collectée, et il n'y a aucune télémétrie. Texte complet : [politique de confidentialité](docs/PRIVACY.md).

## FAQ

**Faut-il une connexion Internet ?**
Seulement pour télécharger les moteurs et les modèles, et pour chercher les mises à jour. La transcription et la rédaction fonctionnent hors ligne.

**Faut-il une carte graphique ?**
Non. Verbatim fonctionne sur le processeur, mais une carte NVIDIA ou AMD accélère nettement la transcription et la rédaction.

**Quelle place et quelle mémoire faut-il ?**
Le pack de base pèse environ 61 Mo. Chaque modèle de transcription fait de 75 Mo à 1,1 Go, et chaque modèle de rédaction de 2,7 à 16 Go environ. Qwen 3.5 4B demande environ 4 Go de mémoire, le 9B environ 8 Go.

**Dans quelle langue sont les comptes-rendus ?**
Dans la langue de la réunion. L'interface de Verbatim est disponible en français et en anglais, au choix à l'installation ou dans les Réglages.

**Verbatim distingue-t-il les intervenants ?**
Pas encore : un seul « Intervenant » est affiché. C'est une piste de la [feuille de route](#feuille-de-route).

**Pourquoi Windows ou mon antivirus affiche-t-il un avertissement ?**
L'installateur et les moteurs ne sont pas encore signés. Voir la section sur l'[antivirus](#si-votre-antivirus-bloque-un-moteur) ; une demande de signature gratuite est en cours.

## Feuille de route

Ce sont des pistes de travail, sans engagement de date ni d'ordre.

**Priorités**
- **Plus de langues d'interface** (espagnol, allemand, italien), et comptes-rendus rédigés dans la langue choisie, indépendamment de celle de la réunion.
- **Séparation des intervenants** (diarisation) : attribuer chaque passage à son locuteur.
- **Whisper en Vulkan** : accélérer la transcription sur les cartes AMD d'avant la série RX 7000 et sur Intel.
- **Signature de code** de l'installateur (demande SignPath en cours), pour supprimer les avertissements de Windows.

**À l'étude**
- **Portage Android** : enregistrement et consultation sur téléphone, selon la faisabilité de l'exécution locale de Whisper et d'un petit modèle de rédaction.
- **Poser des questions à une réunion**, avec renvoi vers le passage concerné.
- **Recherche dans toutes les réunions** et **correction de la transcription** dans l'application.
- **Modèles de comptes-rendus personnalisables** (structure et ton).
- **Transcription en direct** pendant la réunion, et traduction de la transcription.
- **Import de vidéos** et traitement par lots.
- **Enregistrement plus pratique** : raccourci global, icône dans la zone de notification, démarrage proposé à l'ouverture d'une visioconférence.
- **Amélioration audio** avant transcription, **sauvegarde et restauration** des réunions, **catalogue de modèles élargi** (Llama, Mistral, Gemma), **mode ligne de commande**, **macOS et Linux**.

Une idée ou un besoin ? Ouvrez une [issue](https://github.com/Netapick/verbatim/issues).

## Documentation

- [Présentation illustrée](docs/PRESENTATION.md) : le tour de l'application, avec captures d'écran.
- [Documentation technique](docs/DEVELOPMENT.md) : architecture, construction, distribution, mises à jour.
- [Signature du code](docs/SIGNATURE.md) · [Politique de confidentialité](docs/PRIVACY.md)

## Licence

Code sous licence [MIT](LICENSE). Les composants tiers (whisper.cpp, llama.cpp, ROCm) restent sous leurs licences respectives, détaillées dans `THIRD-PARTY-LICENSES.md` de chaque release.

### Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

- Seul l'installateur construit depuis ce dépôt par GitHub Actions (`.github/workflows/release.yml`) est signé ; chaque signature est approuvée manuellement.
- Rôles : [Netapick](https://github.com/Netapick) est auteur, relecteur et approbateur (authentification multifacteur activée).
- Les moteurs tiers (whisper.cpp, llama.cpp) sont téléchargés tels que publiés par leurs auteurs et ne sont pas signés par ce projet.
