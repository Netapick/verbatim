# Verbatim

Français · [English](README.en.md)

**Transcrivez vos réunions et obtenez des comptes-rendus rédigés, sans que rien ne quitte votre ordinateur.**

[![Release](https://img.shields.io/github/v/release/Netapick/verbatim)](https://github.com/Netapick/verbatim/releases/latest) [![Licence MIT](https://img.shields.io/github/license/Netapick/verbatim)](LICENSE) [![Téléchargements](https://img.shields.io/github/downloads/Netapick/verbatim/total)](https://github.com/Netapick/verbatim/releases) ![Windows x64](https://img.shields.io/badge/Windows-x64-0078d4) ![NVIDIA CUDA](https://img.shields.io/badge/NVIDIA-CUDA-76B900?logo=nvidia&logoColor=white) ![AMD ROCm](https://img.shields.io/badge/AMD-ROCm%20%2B%20Vulkan-ED1C24?logo=amd&logoColor=white)

[**Télécharger Verbatim**](https://github.com/Netapick/verbatim/releases/latest) · [Présentation illustrée](docs/PRESENTATION.md) · [Confidentialité](docs/PRIVACY.md)

![Compte-rendu d'une réunion](docs/screenshots/03-compte-rendu.png)

Verbatim est une application de bureau Windows. Elle enregistre ou importe vos réunions, les transcrit avec **Whisper**, puis rédige le compte-rendu avec une IA **Qwen** qui tourne sur votre propre machine. L'audio, les transcriptions et les modèles restent chez vous : aucun compte, aucun abonnement, aucune télémétrie.

## Fonctionnalités

**Enregistrer ou importer**
- Enregistrement du micro, du son du PC (visio, vidéo) ou des deux, avec pause, niveaux en direct et reprise après une coupure.
- Import d'audio (mp3, m4a/aac, ogg, flac, wav…) ou d'une transcription existante (txt, srt, vtt, md).

**Transcrire**
- Whisper local (whisper.cpp), avec détection de parole pour limiter les hallucinations sur les silences.
- Français, anglais, espagnol, allemand, italien, ou détection automatique ; glossaire de sigles transmis au modèle.

**Rédiger**
- Résumé, points abordés, décisions, actions et prochaines étapes, par une IA locale (Qwen 3.5 4B/9B, Qwen 3.8 27B, ou vos propres modèles GGUF). Ollama et une API compatible OpenAI sont possibles en option.
- Sans modèle de rédaction, un résumé simplifié est construit à partir de phrases réelles de la transcription, sans rien inventer.

**Préparer et retrouver**
- Préparations de réunion (objectif, ordre du jour, questions), types de réunion personnalisables.
- Lecteur audio synchronisé avec la transcription (un clic sur un passage lance la lecture), export en texte, sous-titres, JSON, Markdown et audio.
- Thème clair et sombre, interface en français et en anglais, mises à jour automatiques.

## Matériel pris en charge

| Carte graphique | Transcription (Whisper) | Rédaction (Qwen) |
|---|---|---|
| **NVIDIA** | CUDA | CUDA, sinon Vulkan |
| **AMD Radeon RX 7000 / PRO W7000** (RDNA 3) | ROCm | ROCm, sinon Vulkan |
| Autres AMD, Intel | Processeur | Vulkan |
| Sans carte graphique | Processeur | Processeur |

whisper.cpp n'a pas de binaire officiel pour les cartes AMD sous Windows : Verbatim embarque un build ROCm compilé pour les Radeon RX 7000, ce qui rend possible la transcription accélérée sur AMD en plus de NVIDIA. Le mode « Automatique » choisit le moteur le plus rapide disponible, avec repli sur le processeur.

Mesure de la rédaction (RX 7800 XT, Qwen 3.5 4B Q4, lecture / écriture en tokens par seconde) : processeur 94 / 11, Vulkan 2566 / 127, ROCm 3673 / 117.

## Installation

1. Téléchargez `Verbatim_X.Y.Z_x64-setup.exe` depuis la [dernière release](https://github.com/Netapick/verbatim/releases/latest). L'installateur est léger (environ 3 Mo) et propose le français ou l'anglais.
2. Il télécharge seul les moteurs : le pack de base (≈ 61 Mo), puis l'accélération adaptée à votre carte (ROCm ≈ 430 Mo, ou CUDA ≈ 1,3 Go). Aucune question n'est posée, et aucun droit administrateur n'est nécessaire.
3. Lancez Verbatim, puis téléchargez un modèle de transcription et, si vous le souhaitez, un modèle de rédaction depuis **Réglages**. Aucun modèle n'est livré avec l'application.

Les moteurs et modèles sont rangés dans un dossier `ia` à côté de l'application. Si vous avez déjà des modèles (Ollama, fichiers `.gguf` ou `ggml-*.bin`), **Réglages › Modèles** les détecte et les **copie** dans `ia`, sans toucher aux originaux.

**Mises à jour :** au démarrage, Verbatim cherche la dernière version sur GitHub, vérifie son SHA-256 puis l'installe et se relance. Jamais pendant un enregistrement, une transcription ou une rédaction. Désactivable dans **Réglages › Connexions**.

### Si votre antivirus bloque un moteur

Les moteurs tiers (whisper.cpp, llama.cpp) ne sont pas signés : certains antivirus, dont Windows Defender, peuvent les supprimer ou les bloquer (erreur « accès refusé » ou « connexion fermée »). Ajoutez le dossier d'installation de Verbatim (celui qui contient `ia`) aux exclusions : *Sécurité Windows › Protection contre les virus et menaces › Gérer les paramètres › Exclusions › Ajouter une exclusion › Dossier*. Restaurez ensuite les fichiers dans l'historique de protection, ou relancez le téléchargement. Lancer l'application en administrateur n'aide pas.

## Confidentialité

L'audio, les transcriptions et les comptes-rendus sont traités **uniquement sur votre ordinateur**. Les seules connexions sortantes : le téléchargement des moteurs et modèles (GitHub, Hugging Face) à votre demande, la recherche de mise à jour auprès de l'API GitHub, et, seulement si vous la configurez vous-même, une API distante pour la rédaction. Texte complet : [docs/PRIVACY.md](docs/PRIVACY.md).

## Limites

- Windows x64 uniquement.
- Pas de séparation des intervenants pour l'instant : un seul « Intervenant ».
- Les comptes-rendus sont rédigés dans la langue de la réunion (les consignes données à l'IA sont en français).

## Documentation

- [Présentation illustrée](docs/PRESENTATION.md) : le tour de l'application, avec captures d'écran.
- [Documentation technique](docs/DEVELOPMENT.md) : architecture, construction, distribution, mises à jour.
- [Signature du code](docs/SIGNATURE.md) · [Confidentialité](docs/PRIVACY.md)

## Licence

Code sous licence [MIT](LICENSE). Les composants tiers (whisper.cpp, llama.cpp, ROCm) restent sous leurs licences respectives, détaillées dans `THIRD-PARTY-LICENSES.md` de chaque release.

### Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

- Seul l'installateur construit depuis ce dépôt par GitHub Actions (`.github/workflows/release.yml`) est signé ; chaque signature est approuvée manuellement.
- Rôles : [Netapick](https://github.com/Netapick) est auteur, relecteur et approbateur (authentification multifacteur activée).
- Les moteurs tiers (whisper.cpp, llama.cpp) sont téléchargés tels que publiés par leurs auteurs et ne sont pas signés par ce projet.
