# Verbatim

**Transcrivez vos réunions et obtenez des comptes-rendus rédigés, sans que rien ne quitte votre ordinateur.**

Verbatim est une application de bureau Windows. Elle enregistre ou importe vos réunions, les transcrit avec Whisper, puis rédige le compte-rendu avec une IA qui tourne sur votre propre machine : l'audio, les transcriptions et les modèles restent chez vous. L'application n'envoie rien en ligne, sauf si vous configurez vous-même une API distante.

[**Télécharger l'installateur**](https://github.com/Netapick/verbatim/releases/latest) · Windows x64

![Liste des réunions](screenshots/01-reunions.png)

> Les captures de ce document utilisent des réunions fictives.

---

## Pourquoi Verbatim ?

| | |
|---|---|
| **100 % local** | Whisper (transcription) et Qwen via llama.cpp (rédaction) s'exécutent sur votre PC. Aucun compte, aucun abonnement, aucun envoi de données. |
| **Du son au compte-rendu** | Enregistrement ou import audio, transcription minutée, puis résumé, points abordés, décisions, actions et prochaines étapes. |
| **Accéléré par votre carte graphique** | GPU AMD (ROCm, Vulkan) ou NVIDIA (CUDA, Vulkan), avec repli automatique sur le processeur. |
| **Pensé pour les réunions** | Préparations (ordre du jour, questions), glossaire de sigles métier, lecteur audio synchronisé avec la transcription. |

---

## Enregistrer ou importer

Un clic sur **● Enregistrer** capture le **microphone**, le **son du PC** (visio, vidéo, via la boucle WASAPI) ou les deux, mixés en mono. Les niveaux s'affichent en direct, avec pause/reprise et réglage de sensibilité.

![Nouvel enregistrement](screenshots/02-enregistrement.png)

- L'audio est écrit sur disque au fil de l'eau : après un plantage ou une coupure, l'enregistrement est reconstitué au prochain lancement (perte d'environ 2 secondes au plus).
- Vous pouvez aussi **importer un audio** (mp3, m4a/aac, ogg, flac, wav…) ou **une transcription existante** (txt, srt, vtt, md).
- La transcription démarre automatiquement, avec la progression en pourcentage et le matériel utilisé (CPU ou GPU).

---

## Compte-rendu et transcription

Chaque réunion regroupe un lecteur audio (de 0,75× à 2×), le compte-rendu et la transcription minutée. Un clic sur un passage de la transcription lance la lecture à cet instant, et le passage en cours est surligné.

![Compte-rendu d'une réunion](screenshots/03-compte-rendu.png)

- **Résumé**, **points abordés**, **décisions**, **actions** et **prochaines étapes**, générés par l'IA locale.
- Les longues réunions sont découpées en parties, résumées, puis fusionnées. Le glossaire et la préparation liée sont fournis au modèle.
- **Export** : texte, sous-titres (.srt, .vtt), données (.json), compte-rendu (.md) et audio d'origine.
- Sans modèle de rédaction ni API, Verbatim produit un résumé simplifié à partir de phrases réelles de la transcription, sans rien inventer.

---

## Préparer ses réunions

Pour chaque réunion, générez un plan : **objectif**, **ordre du jour chronométré** et **questions à poser**. Les types de réunion (comité budgétaire, lancement de projet, entretien…) sont personnalisables, avec une consigne propre pour guider l'IA.

![Préparations](screenshots/04-preparations.png)

![Détail d'une préparation](screenshots/05-preparation-detail.png)

Une préparation peut être liée à une réunion : elle est alors fournie à l'IA au moment de rédiger le compte-rendu.

---

## Un glossaire pour vos sigles

Ajoutez vos sigles et termes métier : ils sont transmis à Whisper comme vocabulaire initial et au modèle de rédaction, pour limiter les erreurs sur les noms propres et les acronymes.

![Glossaire](screenshots/06-glossaire.png)

---

## Réglages : vos modèles, votre matériel

**Transcription** : choix de la langue (français, anglais, espagnol, allemand, italien ou détection automatique) et du modèle Whisper, du plus léger (Tiny, 75 Mo) au plus précis (Large v3, avec des versions compressées). Mode « Automatique » : GPU quand il est disponible, repli sur le CPU en cas de problème.

![Réglages de transcription](screenshots/07-reglages-transcription.png)

**Compte-rendu** : IA locale **Qwen 3.5** (4B ou 9B) et **Qwen 3.8 27B**, exécutée par llama.cpp, avec calcul sur CPU, Vulkan ou ROCm. Le modèle n'est chargé en mémoire que pendant la rédaction. Vous pouvez aussi réutiliser des modèles déjà présents (GGUF, Ollama) : ils sont copiés dans le dossier de Verbatim, les originaux ne sont pas modifiés.

![Réglages de rédaction](screenshots/08-reglages-redaction.png)

---

## Thème clair et sombre

Un bouton en bas à gauche bascule entre les deux thèmes ; par défaut, celui du système est utilisé.

![Mode sombre](screenshots/09-mode-sombre.png)

![Liste des réunions en mode sombre](screenshots/10-reunions-sombre.png)

---

## Installation

1. Téléchargez `Verbatim_0.1.0_x64-setup.exe` depuis la [page des releases](https://github.com/Netapick/verbatim/releases/latest) (installateur léger, environ 3 Mo, en français).
2. L'installateur télécharge seul, sans rien demander, les moteurs de base (≈ 61 Mo : Whisper et llama.cpp, avec Vulkan). Il détecte aussi votre carte graphique et y ajoute l'accélération adaptée : ROCm (≈ 430 Mo) pour une Radeon RX 7000, CUDA (≈ 685 Mo) pour une carte NVIDIA. Sans carte compatible, Verbatim utilise le processeur.
3. Lancez Verbatim, puis téléchargez un modèle de transcription et, si vous le souhaitez, un modèle de rédaction depuis les **Réglages**. Aucun modèle n'est livré avec l'application.

L'installation se fait par utilisateur, sans droits administrateur. Les moteurs et modèles sont rangés dans un dossier `ia` à côté de l'application.

## En bref

- Windows x64 · interface en français
- Whisper (whisper.cpp) pour la transcription, Qwen (llama.cpp) pour la rédaction
- Pas de diarisation pour l'instant : un seul « Intervenant »
- Composants tiers sous leurs licences respectives (whisper.cpp, llama.cpp, ROCm) : voir `THIRD-PARTY-LICENSES.md` dans la release

Code source et documentation technique : [README](../README.md).
