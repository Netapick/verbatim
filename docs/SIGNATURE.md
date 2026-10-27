# Signature

Français · [English](SIGNATURE.en.md)

## État

- **Signature de code Windows** (certificat) : **aucune pour l'instant**. La demande à [SignPath Foundation](https://signpath.org/) a été refusée le 23/10/2026. Windows peut donc afficher un avertissement à l'installation. Une autre solution reste à choisir (nouvelle demande plus tard, certificat Certum open source, Azure Trusted Signing, Microsoft Store…).
- **Signature des mises à jour** (minisign) : **oui**. Chaque release contient l'installateur, sa signature `.sig` et `latest.json`. L'application vérifie cette signature avec la clé publique de `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`) et refuse une mise à jour invalide. Elle ne remplace pas la signature de code : elle garantit que la mise à jour vient bien du projet.
- **Moteurs tiers** (whisper.cpp, llama.cpp) : téléchargés tels que publiés par leurs auteurs, non signés par ce projet. Les antivirus peuvent les bloquer : voir « Si votre antivirus bloque un moteur » dans le [README](../README.md).

## Vérifier un téléchargement

Les notes de chaque release donnent la **somme SHA-256** de l'installateur. Pour vérifier que le fichier téléchargé est identique à celui construit par GitHub Actions (PowerShell, dans le dossier du fichier) :

```powershell
Get-FileHash .\Verbatim_X.Y.Z_x64-setup.exe -Algorithm SHA256
```

Comparez le résultat à la somme des notes. Cela détecte un fichier corrompu ou altéré en route ; ce n'est pas une signature de code (voir « État » ci-dessus). Les packs de moteurs téléchargés par l'installateur sont, eux, vérifiés automatiquement par SHA-256 (voir `packs.nsh` dans [DEVELOPMENT.md](DEVELOPMENT.md)). Les sommes ne figurent que sur les releases construites après leur ajout (à partir de la prochaine version).

## Clé de signature des mises à jour

1. Génération (une seule fois) : `npx tauri signer generate -w .secrets/verbatim-updater.key`. Le dossier `.secrets/` et les fichiers `*.key` sont ignorés par git : **ne jamais commiter la clé privée**.
2. Clé publique : dans `src-tauri/tauri.conf.json`, `plugins.updater.pubkey`.
3. Clé privée : dans le secret GitHub `TAURI_SIGNING_PRIVATE_KEY` (Settings › Secrets and variables › Actions), plus une copie de sauvegarde hors du dépôt. Si la clé a un mot de passe : secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. Perdre la clé privée oblige à republier l'application avec une nouvelle clé publique : les versions déjà installées ne pourraient plus se mettre à jour seules.

## Publier une version

Voir « Publier une version » dans [DEVELOPMENT.md](DEVELOPMENT.md) : monter la version, mettre à jour `CHANGELOG.md`, puis `git tag vX.Y.Z && git push origin vX.Y.Z`. `.github/workflows/release.yml` construit, signe la mise à jour et publie la release.
