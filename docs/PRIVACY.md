# Politique de confidentialité — Verbatim

*Privacy policy — English summary at the end.*

Dernière mise à jour : 18 octobre 2026.

## Vos données restent sur votre ordinateur
Verbatim enregistre, transcrit et résume vos réunions **localement**. L'audio, les transcriptions, les comptes-rendus, les préparations et le glossaire sont stockés sur votre ordinateur, dans le dossier de l'application. Le projet ne collecte aucune donnée personnelle, n'a pas de serveur et n'intègre aucune télémétrie, aucun suivi ni aucune publicité.

## Les seules connexions réseau de l'application
1. **Téléchargement des moteurs et des modèles**, uniquement quand vous le demandez (ou pendant l'installation) : GitHub (moteurs whisper.cpp et llama.cpp, packs de l'installateur) et Hugging Face (modèles Whisper et Qwen). Ces services voient une requête de téléchargement standard, comme avec un navigateur.
2. **Recherche de mise à jour** : au démarrage, l'application interroge l'API GitHub (`releases/latest` du dépôt `Netapick/verbatim`) et, si une version plus récente existe, télécharge l'installateur et vérifie son empreinte SHA-256. Aucune donnée vous concernant n'est envoyée au-delà de ce qu'une requête HTTPS contient (adresse IP, version de l'application dans l'en-tête `User-Agent`). Vous pouvez désactiver ces mises à jour dans **Réglages › Connexions**.
3. **API distante de rédaction (optionnelle)** : si, et seulement si, vous configurez vous-même une adresse d'API et une clé dans les Réglages, le texte des transcriptions est envoyé à ce service pour rédiger le compte-rendu. Son traitement relève alors de la politique de confidentialité de ce fournisseur. Sans configuration, rien n'est envoyé.

## Composants tiers
Verbatim s'appuie sur whisper.cpp et llama.cpp (exécutés localement) et sur des modèles téléchargés depuis Hugging Face. Ils s'exécutent sur votre machine ; leurs licences respectives s'appliquent (voir `THIRD-PARTY-LICENSES.md` dans les releases).

## Suppression de vos données
Désinstaller l'application supprime ses fichiers de programme ; le dossier `ia` (modèles) peut être conservé ou supprimé à la désinstallation. Vos réunions se suppriment depuis l'application (Corbeille) ou en supprimant son dossier de données.

## Contact
Pour toute question : ouvrez une [issue](https://github.com/Netapick/verbatim/issues) sur le dépôt.

---

## English summary
Verbatim records, transcribes and summarizes meetings entirely on your computer. It collects no personal data and includes no telemetry. Its only network activity is: downloading engines and models from GitHub and Hugging Face when you ask for it, checking GitHub for updates at startup (can be turned off in Settings › Connections), and — only if you configure one yourself — sending transcripts to a remote API of your choice for report writing.
