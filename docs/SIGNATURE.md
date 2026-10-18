# Signature du code (SignPath)

L'installateur est signé par [SignPath Foundation](https://signpath.org/) (gratuit pour les projets open source, licence MIT du dépôt). La signature se fait sur GitHub Actions : `.github/workflows/release.yml` construit l'installateur, l'envoie à SignPath, récupère la version signée et publie la release.

## Ce qui est signé, et ce qui ne l'est pas
- **Signé** : l'installateur `Verbatim_X.Y.Z_x64-setup.exe`.
- **Non signé** : les moteurs tiers téléchargés (whisper.cpp, llama.cpp). SignPath ne signe que les fichiers du projet. Les antivirus peuvent donc encore les bloquer : voir « Si l'antivirus bloque un moteur » dans la présentation.

## Mise en place (une seule fois, à faire par le propriétaire du dépôt)
1. Faire la demande sur <https://signpath.org/apply> pour le dépôt `Netapick/verbatim` (licence MIT, dépôt public). La validation est manuelle.
2. Dans SignPath, créer le projet de slug `verbatim`, avec :
   - une **configuration d'artefact** : un fichier zip contenant `*_x64-setup.exe` (type PE, à signer) ;
   - une **politique de signature** de slug `release-signing`, liée au dépôt GitHub (vérification d'origine).
3. Installer l'application GitHub SignPath sur le dépôt, puis ajouter dans GitHub (Settings › Secrets and variables › Actions) :
   - secret `SIGNPATH_API_TOKEN` : jeton d'API d'un utilisateur CI de SignPath ;
   - variable `SIGNPATH_ORGANIZATION_ID` : identifiant de l'organisation SignPath.

## Publier une version signée
1. Monter la version dans `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, puis commiter.
2. `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. Le workflow publie la release avec l'installateur signé. La mise à jour automatique de l'app lit cette release (`releases/latest`) et vérifie son SHA-256.

Remarque : le dépôt ne publie plus d'exe construit localement ; les anciennes versions (0.1.0 à 0.2.1) restent non signées.
