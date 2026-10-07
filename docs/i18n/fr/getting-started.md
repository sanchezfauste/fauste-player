# Premiers pas

## Installation {#install}

Téléchargez un paquet pour votre système depuis la page **Releases** du
projet. Chaque fichier a un `.sha256` à côté de lui. Pour vérifier un
téléchargement, exécutez `sha256sum -c <file>.sha256` sous Linux, ou
`shasum -a 256 -c <file>.sha256` sous macOS.

### Linux {#linux}

| Paquet | Installation | Mise à jour | Suppression |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | installez le `.deb` plus récent de la même façon | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | installez le `.rpm` plus récent | `sudo dnf remove fauste-player` |
| AppImage (toute distribution) | `chmod +x fauste-player-<version>-x86_64.AppImage`, puis lancez-le | remplacez le fichier | supprimez le fichier |
| Bundle Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | installez le bundle plus récent | `flatpak uninstall org.fauste.FaustePlayer` |
| Archive | extrayez-la et lancez `fauste-player` | extrayez la plus récente | supprimez le dossier |

- **Intégration au bureau :** les paquets ajoutent **Fauste Player** au menu
  des applications et vous permettent d'ouvrir avec lui des playlists
  `.m3u`, `.m3u8` et `.pls`, qui sont alors importées comme nouvelles
  playlists.
- **Bibliothèques nécessaires :** les bibliothèques ALSA et D-Bus. Tous les
  environnements de bureau les ont, et les paquets les déclarent. JACK est
  utilisé s'il est installé, et n'est jamais obligatoire.
- **AppImage :** s'il ne démarre pas parce que FUSE manque, lancez-le avec
  `--appimage-extract-and-run`.
- **Flatpak :**
  - Le bundle a besoin du runtime freedesktop de Flathub. Si le dépôt Flathub
    n'est pas configuré, exécutez d'abord
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - Le bac à sable peut lire votre dossier personnel (pour lire votre
    musique là où elle se trouve).
  - Il joue via PulseAudio, ou directement via ALSA pour les périphériques
    bit-perfect.
- **Bibliothèques de bureau :** la fenêtre utilise libxkbcommon et EGL ou
  OpenGL, avec Wayland ou X11. Tous les environnements de bureau les ont, et
  le `.deb` et le `.rpm` les déclarent. Sur un système très minimal,
  installez-les avant d'utiliser l'AppImage.

### Windows {#windows}

Exécutez `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Il s'installe
pour tous les utilisateurs dans *Program Files* et ajoute une entrée au menu
Démarrer.
- **Mise à jour :** exécutez le nouvel installateur. Il remplace la version
  installée.
- **Suppression :** utilisez **Paramètres → Applications**.
- **Installateur non signé :** si la version n'est pas signée, SmartScreen
  avertit d'un éditeur inconnu. Choisissez **Informations complémentaires →
  Exécuter quand même**.

L'archive `.zip` est une alternative portable : extrayez-la et lancez
`fauste-player.exe`.

### macOS {#macos}

Ouvrez `fauste-player-<version>-macos-universal.dmg` et faites glisser
**Fauste Player** dans **Applications**. La même application fonctionne sur
Apple silicon et sur Intel (macOS 11 ou ultérieur).
- **Mise à jour :** remplacez l'application de la même façon.
- **Suppression :** déplacez-la dans la Corbeille.
- **Application non signée :** si la version n'est pas signée et notariée,
  macOS refuse le premier démarrage.
  - macOS 15 et ultérieur : ouvrez **Réglages Système → Confidentialité et
    sécurité**, faites défiler jusqu'au message concernant Fauste Player,
    choisissez **Ouvrir quand même**, puis confirmez.
  - macOS 14 et antérieur : faites un clic droit sur l'application, choisissez
    **Ouvrir**, puis confirmez.
  - Ou exécutez `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`.
- **JACK sur macOS :** une application signée et notariée ne peut charger
  qu'une bibliothèque JACK elle-même signée. Sinon, JACK apparaît comme
  indisponible ; utilisez Core Audio.
- **Playlists :** sous macOS, elles s'importent depuis Paramètres →
  Playlists. Ouvrir un fichier de playlist avec l'application depuis le
  Finder n'est pas pris en charge.

### Une seule instance à la fois {#one-instance-at-a-time}

Un seul Fauste Player s'exécute par dossier de données. Ouvrir une playlist
depuis le gestionnaire de fichiers pendant qu'il tourne importe cette
playlist dans l'application en cours d'exécution. Le relancer sans playlist
affiche un message indiquant qu'il est déjà en cours d'exécution. Pour
exécuter des instances séparées côte à côte (par exemple deux studios sur un
seul ordinateur), donnez à chacune son propre dossier avec `FAUSTE_HOME`.

### Ligne de commande {#command-line}

`fauste-player --version` affiche la version. La barre de titre de la fenêtre
montre le nom et la version, et le bouton **À propos** (une icône
d'information, à gauche de **Paramètres**) ouvre la fenêtre **À propos de
Fauste Player**, avec le copyright et les mentions de licence (**Licences
tierces** ouvre le fichier des mentions installé avec les paquets de
publication). Dans une langue traduite par IA, la fenêtre À propos indique
aussi que la traduction peut contenir des erreurs. Fermez-la avec **Fermer**
ou `Esc`. `fauste-player --help` liste les options. Les fichiers de
playlist passés en arguments sont importés comme nouvelles playlists.

![La fenêtre À propos : version, copyright et licences des composants inclus](../../images/guide/about.png)

Sous Windows, le programme de publication n'ouvre pas de fenêtre de
console : `--version`, `--help` et les erreurs de démarrage apparaissent
plutôt dans une boîte de message.

Quand un réglage demande un redémarrage, une pastille **Redémarrage en
attente** apparaît dans la barre supérieure : appuyez dessus pour redémarrer
(voir [Paramètres](settings.md#restart-pending)).

### Fermer pendant que de l'audio est à l'antenne {#closing-while-audio-is-on-air}

Fermer la fenêtre pendant que quelque chose est à l'antenne ne quitte pas
l'application. La fenêtre passe au premier plan (même si elle était réduite)
et une boîte de dialogue **De l'audio est à l'antenne** liste ce qui sonne :
les lecteurs en lecture ou en pause (`P1 — titre`) et les cartouches en cours
de lecture avec leur numéro dans la page (`Cartouche 3 — titre`). Le CUE d'un
lecteur ou du mur de cartouches ne compte pas. Choisissez **Annuler** (ou
`Esc`, ou cliquez en dehors de la boîte de dialogue) pour continuer à
jouer, ou **Arrêter et fermer** pour arrêter tous les lecteurs à l'antenne et
toutes les cartouches, puis quitter. La session est enregistrée comme à
n'importe quelle autre sortie. Quand rien n'est à l'antenne, la fenêtre se
ferme aussitôt. La boîte de dialogue a priorité sur les Paramètres et sur la
fenêtre À propos, et les raccourcis clavier ne font rien tant qu'elle est
ouverte (les commandes MIDI et à distance agissent toujours).

## Premier démarrage {#first-start}

La fenêtre s'ouvre avec quatre lecteurs, chacun affichant une playlist vide.
Rien ne joue tant que vous n'appuyez pas sur PLAY : c'est aussi vrai après un
redémarrage ou un plantage.

## Ajouter de la musique {#add-music}

- Cliquez sur **+ Ajouter** en bas d'un lecteur et choisissez des fichiers,
  ou
- faites glisser des fichiers audio ou des dossiers depuis votre gestionnaire
  de fichiers vers une liste de pistes. Les dossiers ajoutent les fichiers
  audio qu'ils contiennent directement, pas ceux de leurs sous-dossiers.

Formats pris en charge : WAV, AIFF, CAF, FLAC, MP3 (et MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF et DFF) et audio Matroska (MKA).
Les fichiers DSD sont lus convertis en PCM à leur fréquence divisée par 32
(88,2 kHz pour le DSD64), ou intacts en DoP ou en DSD natif sur un
périphérique bit-perfect configuré pour cela (voir
[Sortie bit-perfect](bit-perfect.md#dsd)). L'Opus est toujours lu à 48 kHz.
Les fichiers WavPack doivent être sans perte et mono ou stéréo ; les WavPack
hybrides et multicanaux apparaissent comme illisibles, tout comme les
fichiers DFF compressés en DST.

Chaque fichier est analysé en arrière-plan. L'analyse lit le titre,
l'artiste, l'album et la pochette, dessine la forme d'onde, et trouve où le
son commence et finit et où mixer vers la piste suivante. Vous pouvez lire
une piste avant la fin de son analyse.

## Lire {#play}

- Appuyez sur **PLAY** (ou sur la touche numérique du lecteur, `1` pour P1)
  pour démarrer la piste **suivante**, marquée en vert dans la liste.
- Appuyez de nouveau sur **PLAY** pendant qu'une piste est à l'antenne pour
  la fondre dans la suivante.
- **Double-cliquez** sur une piste pour en faire la suivante.

En mode **CONT** (continu), le lecteur mixe lui-même vers la piste suivante
au point MIX. En mode **SINGLE**, il s'arrête à la fin de chaque piste.
Poursuivez avec [Lecteurs](players.md).
