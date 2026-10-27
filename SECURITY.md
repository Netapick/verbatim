# Politique de sécurité

Français · [English below](#security-policy)

## Signaler une faille

**Ne publiez pas une faille dans une issue publique.** Utilisez le signalement privé de GitHub : onglet [Security](https://github.com/Netapick/verbatim/security) du dépôt, puis **Report a vulnerability**. Seuls les mainteneurs voient le message.

Décrivez ce que vous avez observé, la version de Verbatim concernée (affichée en bas à gauche de l'application) et, si possible, les étapes pour reproduire le problème. Une réponse est donnée dès que possible ; ce projet est maintenu par une seule personne, sans délai garanti.

Si le signalement privé n'est pas disponible, ouvrez une [issue](https://github.com/Netapick/verbatim/issues) qui dit seulement qu'un signalement de sécurité est à faire, **sans détail technique** : un contact privé sera alors convenu.

## Versions prises en charge

Seule la **dernière version publiée** reçoit des correctifs. L'application propose la mise à jour au démarrage.

## Ce qui est concerné

Verbatim fonctionne entièrement en local : il n'a pas de serveur et ne collecte aucune donnée ([confidentialité](docs/PRIVACY.md)). Les sujets les plus utiles à signaler :

- la **mise à jour** de l'application (signature, téléchargement, installation) ;
- le **téléchargement des moteurs et des modèles** (vérification des sommes SHA-256, remplacement de fichiers) ;
- les **serveurs locaux** lancés par l'application (`llama-server`, `whisper-server`, écoute sur `127.0.0.1` uniquement) ;
- le stockage des **réglages**, dont la clé de l'API distante si vous en configurez une ;
- l'**installateur** et sa désinstallation.

## Hors périmètre

- L'absence de signature de code Windows de l'installateur : elle est connue et expliquée dans [docs/SIGNATURE.md](docs/SIGNATURE.md).
- Les alertes d'antivirus sur les moteurs tiers : voir [docs/CODES.md](docs/CODES.md#antivirus-la-cause-la-plus-fréquente).
- Les failles de whisper.cpp, de llama.cpp ou des modèles eux-mêmes : à signaler à leurs projets respectifs. Signalez ici seulement ce qui tient à la façon dont Verbatim les utilise.

---

## Security policy

### Reporting a vulnerability

**Please do not disclose a vulnerability in a public issue.** Use GitHub's private reporting: the repository's [Security](https://github.com/Netapick/verbatim/security) tab, then **Report a vulnerability**. Only the maintainers can see the message.

Describe what you observed, the Verbatim version concerned (shown at the bottom left of the application) and, if possible, the steps to reproduce it. A reply is given as soon as possible; the project is maintained by one person, with no guaranteed delay.

If private reporting is not available, open an [issue](https://github.com/Netapick/verbatim/issues) saying only that a security report is pending, **with no technical detail**: a private channel will then be arranged.

### Supported versions

Only the **latest published version** receives fixes. The application offers the update at startup.

### Scope

Verbatim runs entirely locally: it has no server and collects no data ([privacy](docs/PRIVACY.en.md)). The most useful topics to report:

- the application **update** (signature, download, installation);
- the **download of engines and models** (SHA-256 checks, file replacement);
- the **local servers** started by the application (`llama-server`, `whisper-server`, listening on `127.0.0.1` only);
- the storage of **settings**, including the remote API key if you configure one;
- the **installer** and its uninstallation.

### Out of scope

- The installer's lack of a Windows code signature: it is known and explained in [docs/SIGNATURE.en.md](docs/SIGNATURE.en.md).
- Antivirus alerts on third-party engines: see [docs/CODES.en.md](docs/CODES.en.md#antivirus-the-most-common-cause).
- Vulnerabilities in whisper.cpp, llama.cpp or the models themselves: report them to their own projects. Report here only what comes from how Verbatim uses them.
