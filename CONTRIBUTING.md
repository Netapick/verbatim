# Contribuer à Verbatim

Français · [English below](#contributing-to-verbatim)

Merci de votre intérêt. Verbatim est un projet personnel, maintenu par une seule personne : les retours sont les bienvenus, mais les réponses peuvent prendre du temps.

## Signaler un problème

Ouvrez une [issue](https://github.com/Netapick/verbatim/issues/new/choose) avec le modèle « Signaler un bug ». Les informations qui font gagner le plus de temps :

- la **version** de Verbatim (en bas à gauche de l'application) et de Windows ;
- la **carte graphique** et le moteur choisi (Réglages › Modèles) ;
- le **message affiché**, tel quel : [docs/CODES.md](docs/CODES.md) explique la plupart des codes et des messages ;
- en cas d'échec d'installation, le fichier **`redist-install.log`**, à la racine du dossier d'installation.

**Retirez de ces fichiers tout ce qui est privé** (noms, contenu de réunion) avant de les publier. Ne joignez jamais d'enregistrement réel.

Pour une **faille de sécurité**, suivez plutôt [SECURITY.md](SECURITY.md).

## Proposer une idée

Ouvrez une issue avec le modèle « Proposer une idée ». Avant, regardez la [feuille de route](README.md#feuille-de-route) : beaucoup de pistes y figurent déjà, et votre vote ou votre cas d'usage aide à les ordonner. Verbatim reste **100 % local et sans télémétrie** : une idée qui demande d'envoyer des données à un service ne sera pas retenue par défaut.

## Proposer du code

1. Ouvrez d'abord une issue pour les changements importants, afin de vérifier qu'ils s'accordent avec le projet.
2. Compilez et lancez l'application en suivant [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) (section « Construire »). Windows x64 uniquement pour l'instant.
3. Gardez les changements ciblés : un sujet par *pull request*, dans le style du code voisin (interface en JavaScript sans framework, côté Rust dans `src-tauri/src/`).
4. Testez ce que vous modifiez dans l'application réelle (`npx tauri dev`) et dites dans la *pull request* ce que vous avez essayé.
5. Si vous touchez au texte affiché, mettez à jour la traduction anglaise (`ui/i18n.js`). Si vous touchez à la documentation, mettez à jour les deux langues (`*.md` et `*.en.md`).

Les messages de commit sont en français, courts, à l'indicatif ou au nom d'action : « Pastille d'enregistrement : déplacement sans barre de titre ».

En contribuant, vous acceptez que votre code soit publié sous la [licence MIT](LICENSE) du projet.

---

## Contributing to Verbatim

Thanks for your interest. Verbatim is a personal project maintained by one person: feedback is welcome, but replies can take a while.

### Reporting a problem

Open an [issue](https://github.com/Netapick/verbatim/issues/new/choose) with the "Report a bug" template. The details that save the most time:

- the **version** of Verbatim (bottom left of the application) and of Windows;
- the **graphics card** and the engine chosen (Settings › Models);
- the **message shown**, as is: [docs/CODES.en.md](docs/CODES.en.md) explains most codes and messages;
- for a failed installation, the **`redist-install.log`** file, at the root of the installation folder.

**Remove anything private from these files** (names, meeting content) before posting them. Never attach a real recording.

For a **security vulnerability**, follow [SECURITY.md](SECURITY.md#security-policy) instead.

### Suggesting an idea

Open an issue with the "Suggest an idea" template. First look at the [roadmap](README.en.md#roadmap): many leads are already there, and your use case helps order them. Verbatim stays **100% local with no telemetry**: an idea that requires sending data to a service will not be accepted by default.

### Contributing code

1. Open an issue first for significant changes, to check they fit the project.
2. Build and run the application following [docs/DEVELOPMENT.en.md](docs/DEVELOPMENT.en.md) (section "Building"). Windows x64 only for now.
3. Keep changes focused: one topic per pull request, in the style of the surrounding code (framework-free JavaScript interface, Rust side in `src-tauri/src/`).
4. Test what you change in the real application (`npx tauri dev`) and say in the pull request what you tried.
5. If you touch displayed text, update the English translation (`ui/i18n.js`). If you touch the documentation, update both languages (`*.md` and `*.en.md`).

Commit messages are in French, short, as a noun phrase or in the indicative: "Pastille d'enregistrement : déplacement sans barre de titre".

By contributing, you agree that your code is published under the project's [MIT license](LICENSE).
