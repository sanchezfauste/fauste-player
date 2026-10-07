# Playlists

## Onglets {#tabs}

Chaque lecteur a une rangée d'onglets, un par playlist. Tous les lecteurs
voient les mêmes playlists ; chacun choisit laquelle il affiche. Un point sur
un onglet montre où se trouvent les pistes du lecteur : **rouge** pour la
piste à l'antenne, **vert** pour la suivante.

Les onglets se partagent la largeur du lecteur. Un nom qui ne tient pas se
termine par « … » ; survolez l'onglet pour le lire en entier. Avec beaucoup
de playlists, les onglets cessent de rétrécir à une largeur minimale, des
flèches apparaissent aux extrémités de la rangée et la molette de la souris
sur les onglets les fait défiler. L'onglet que vous choisissez, et celui
qu'affiche un lecteur, est ramené dans la vue.

Changer d'onglet ne modifie jamais ce qui est à l'antenne ni ce qui est
suivant. Quand une piste se termine, le lecteur continue dans la playlist qui
contient cette piste.

Les playlists se créent, se renomment et se suppriment dans les
[Paramètres](settings.md). La dernière playlist, et une playlist dont une
piste est à l'antenne, ne peuvent pas être supprimées.

## Le tableau des pistes {#the-track-table}

![Une playlist : pistes jouées grisées, piste à l'antenne en rouge, piste suivante en vert, et le pied de page avec le temps restant](../../images/guide/playlist.png)

| Colonne | Contenu |
|---|---|
| `#` | Position, complétée par des zéros ; une icône la remplace pour les pistes en cours et suivante |
| Titre | Depuis les tags, ou le nom du fichier (`Artiste - Titre.mp3` est découpé). Les icônes de répétition et d'arrêt après d'une piste se trouvent avant |
| Artiste | Depuis les tags ; « Artiste inconnu » quand il n'y en a pas |
| Album | Depuis les tags |
| Date | La date d'enregistrement telle que le fichier la stocke (`2019`, `2019-05` ou `2019-05-14`, avec une heure s'il y en a une) |
| Genre | Depuis les tags |
| Durée | Durée de lecture, du cue-in au cue-out (le fichier entier avec **Utiliser le cue-in et le cue-out** désactivé) |
| Intro | Durée de l'intro, de l'endroit où la piste commence à jouer jusqu'à son marqueur d'intro ; vide quand la piste n'a pas de marqueur d'intro |
| Fichier | Le nom du fichier, avec son extension |

Une nouvelle installation affiche `#`, Titre, Artiste et Durée. Les autres
colonnes sont facultatives ; voir **Choisir les colonnes** plus bas. Une piste
à laquelle une valeur manque affiche une cellule vide, sauf Artiste, qui
affiche « Artiste inconnu ».

Les colonnes remplissent le tableau et gardent leurs proportions quand la
fenêtre est redimensionnée ; les colonnes de texte obtiennent le plus de
place. Faites glisser les séparateurs d'en-tête pour changer les
proportions : les colonnes à droite du séparateur suivent le pointeur à
chaque image (elles se partagent ce qui reste proportionnellement à leur
largeur), celles à sa gauche restent, et les largeurs sont enregistrées quand
vous relâchez. Aucune colonne ne devient plus étroite que son minimum. Les
largeurs sont mémorisées par lecteur ; une colonne que vous affichez plus tard
commence à sa largeur par défaut et les autres gardent leurs proportions.

### Choisir les colonnes {#choosing-the-columns}

Titre et Durée sont toujours affichés. Toute autre colonne peut être affichée
ou masquée, et n'importe quelle colonne, ces deux-là comprises, peut être
déplacée. La liste est la même pour chaque lecteur et chaque playlist, et elle
est enregistrée dans `config.json` sous `ui.table_columns`. Trois façons de la
modifier :

- **Paramètres → Playlists → Colonnes du tableau :** cochez les colonnes à
  afficher ; les flèches font monter ou descendre une colonne affichée (elles
  se lisent de gauche à droite dans les tableaux). **Colonnes par défaut**
  revient à `#`, Titre, Artiste et Durée.
- **Clic droit sur un en-tête :** un menu avec une case à cocher pour chaque
  colonne facultative. Une colonne que vous affichez apparaît à l'extrémité
  droite ; faites-la glisser de là.
- **Faire glisser un en-tête** sur un autre : déposez-le sur la moitié gauche
  d'un en-tête pour placer la colonne avant lui, sur la moitié droite pour la
  placer après. Déposer ailleurs ne fait rien.

Un nom dans `ui.table_columns` que cette version ne connaît pas est ignoré, et
un Titre ou une Durée manquant est rajouté.

À l'ouverture de l'application, chaque tableau défile de sorte que la piste
suivante de son lecteur soit au milieu du tableau (aussi près que les
extrémités de la liste le permettent). Cela n'arrive qu'une fois, au
démarrage, et seulement quand la piste suivante se trouve dans la playlist
que le tableau affiche.

Quand un lecteur passe à une autre piste, son tableau affiche la playlist de
cette piste et fait défiler sa ligne vers le haut, sauf si vous avez utilisé
le tableau dans les 10 dernières secondes (défilement, glissement d'une piste,
ouverture du menu d'une piste ou clic sur un onglet) : il attend alors que
vous l'ayez laissé tranquille pendant ce temps. Ce délai est
`ui.follow_current_grace_secs` dans `config.json` ; 0 désactive le suivi.

Les lecteurs sont indépendants : plusieurs lecteurs peuvent afficher la même
playlist, chacun avec sa propre piste suivante, ses propres marques de lecture
et ses propres temps dans le pied de page. Lire, arrêter ou passer sur un
lecteur ne déplace jamais la suivante d'un autre lecteur. La même piste peut
même être à l'antenne sur deux lecteurs à la fois. Modifier la playlist
(ajouter, déplacer ou retirer des entrées) ou un fichier qui devient illisible
peut quand même changer la suivante de tout lecteur qui l'affiche.

Couleurs des lignes :

| Ligne | Signification |
|---|---|
| **Rouge**, avec une icône de haut-parleur (ou de pause) | À l'antenne sur ce lecteur. Elle peut aussi afficher la flèche verte : la piste à l'antenne est aussi la suivante, elle est donc rejouée |
| **P2** rouge (ou un autre numéro) dans la colonne du numéro | À l'antenne sur ce lecteur-là |
| **Vert**, avec une flèche | La piste suivante de ce lecteur |
| Grisée | Déjà jouée sur ce lecteur |
| Fichier avec une croix / une icône d'avertissement | Fichier introuvable / illisible (il est ignoré) ; survolez la ligne : la bulle commence par la raison, en ambre, puis les champs habituels. Un fichier introuvable est recherché de nouveau toutes les 30 s (`tuning.missing_recheck_ms`). |
| Flèches de rechargement à droite du titre | Analysée par une version antérieure ; elle joue quand même avec cette analyse. **Paramètres → Analyse → Analyser les pistes obsolètes** la met à jour (les pistes sur un lecteur sont mises à jour de toute façon) |
| Sablier à droite du titre | La piste attend son analyse (survolez le sablier : *Analyse en attente*). Elle joue quand même, et le sablier disparaît à la fin de l'analyse |
| Violette | Sélectionnée |

**Infobulle d'une piste.** Survolez une ligne un instant pour voir son titre, son artiste, son album, sa date, son genre, sa durée, son format (type, fréquence d'échantillonnage et profondeur en bits quand elles sont connues) et le chemin de son fichier. Un champ que le fichier n'a pas est omis. La bulle est la seule information au survol d'une ligne, et elle ne bouge jamais tant qu'elle est affichée. Pour un fichier introuvable ou illisible, elle commence par la raison.

## Souris {#mouse}

- Un **clic** sélectionne une piste. Un **double-clic** en fait la piste
  suivante de ce lecteur. Sur la piste à l'antenne, il la fait rejouer quand
  le passage en cours se termine.
- Un **clic droit** ouvre le menu contextuel :

![Le menu contextuel d'une piste](../../images/guide/track-menu.png)

| Élément | Action |
|---|---|
| Lire maintenant | Démarrer cette piste aussitôt (en mixant si le lecteur est à l'antenne) |
| Définir comme suivante | Comme le double-clic. Sur la piste à l'antenne, elle est rejouée, depuis son début, quand le passage en cours se termine (en mixant comme Répéter, sans interruption), puis le lecteur continue. Il n'agit qu'une fois. Stop à la fin, le mode SINGLE et une marque Arrêt après arrêtent quand même d'abord le lecteur. Pendant qu'un CUE tourne, il passe à la nouvelle suivante |
| Pré-écouter sur le CUE | La jouer sur la sortie CUE (cela ouvre la fenêtre CUE). Grisé quand le lecteur n'a pas de sortie Cue distincte de sa sortie Main |
| Modifier les tags… | Ouvrir l'éditeur de tags pour cette piste. **Enregistrer** écrit les modifications dans le fichier audio ; **Annuler** (ou Échap, quand aucun enregistrement n'est en cours) ferme sans écrire. L'élément est grisé, avec la raison au survol, tant que la piste est à l'antenne, sur le CUE ou sur une cartouche en lecture, tant que ses tags n'ont pas été lus, quand le fichier est introuvable, et pour les formats dont les tags ne peuvent pas être écrits (par exemple le DSD) |
| Réanalyser | Analyser cette piste de nouveau maintenant, quel que soit son état. Un fichier corrigé qui était illisible est aussi repris tout seul (voir [Dépannage](troubleshooting.md)). Les marqueurs manuels sont conservés |
| Ajouter des pistes en dessous… | Choisir des fichiers à insérer après cette piste |
| Dupliquer | Insérer une copie non jouée en dessous (avec ses marques de répétition et d'arrêt après) |
| Répéter cette piste | Cochez pour la rejouer encore et encore, sans interruption, jusqu'à ce que vous appuyiez sur PLAY (suivante), Précédente, Stop ou Stop en fondu, ou activiez Stop à la fin. Pause la maintient en répétition. Une icône de répétition s'affiche avant le titre |
| Arrêter après cette piste | Cochez pour arrêter le lecteur quand cette piste se termine, à chaque fois qu'elle est lue (dans n'importe quel mode). Contrairement au bouton **Stop à la fin** du lecteur, la marque reste avec la piste et est enregistrée avec la playlist. L'icône d'arrêt après s'affiche avant le titre. Elle l'emporte sur Répéter |
| Déplacer vers ▸ | La déplacer à la fin d'une autre playlist |
| Retirer de la playlist | La retirer ; impossible tant qu'elle est à l'antenne |

## Modifier les tags {#editing-tags}

**Modifier les tags…** ouvre une fenêtre pour une piste. Tant qu'elle est
ouverte, aucun raccourci clavier n'agit, et les fichiers déposés sur la
fenêtre de l'application sont ignorés.

![L'éditeur de tags d'un fichier FLAC, avec sa pochette, son titre, son artiste, son album, sa date et son genre](../../images/guide/tag-editor.png)

- **Ce que vous voyez.** L'éditeur lit le fichier à son ouverture (il affiche
  « Lecture des tags… » entre-temps). Toujours affichés : titre, artiste,
  album, artiste de l'album, date, numéro de piste et total, numéro de disque
  et total, genre, compositeur et commentaire. Affichés quand le fichier les
  possède : sous-titre, regroupement, BPM, tonalité initiale, ambiance, ISRC,
  éditeur, numéro de catalogue, copyright, artiste original, album original,
  date de sortie originale, parolier, chef d'orchestre, remixeur, arrangeur,
  interprète, langue, encodé par, paroles, tri du titre, tri de l'artiste, tri
  de l'album, tri de l'artiste de l'album, tri du compositeur et site web de
  l'artiste.
- **Ajouter un champ.** Le menu sous les champs liste les autres champs. Il ne
  propose que ce que le format de tags du fichier peut stocker (un WAV avec
  RIFF INFO, un AIFF ou un ancien tag ID3v1 stockent moins de champs
  qu'ID3v2, FLAC ou MP4), et il est grisé quand il n'y a plus rien à ajouter.
  Un des champs toujours affichés que le format ne peut pas stocker est grisé
  avec une note. Vider un champ le retire du fichier ; un champ ajouté laissé
  vide n'est pas écrit.
- **Plusieurs valeurs.** Les champs qui peuvent contenir plusieurs valeurs
  (artiste, artiste de l'album, genre, compositeur, ambiance et les crédits
  comme parolier, chef d'orchestre, remixeur, arrangeur et interprète, et
  langue) affichent une valeur par ligne ; **Enregistrer** écrit une valeur
  par ligne à la manière propre au format. Le commentaire et les paroles sont
  du texte libre sur plusieurs lignes.
- **Vérifications.** La date et la date de sortie originale suivent l'ISO 8601
  (`2019`, `2019-05` ou `2019-05-14`, éventuellement avec une heure) ; le
  numéro de piste et de disque, leurs totaux et le BPM sont des nombres
  entiers, et un total exige son numéro. Un champ à la valeur invalide est
  marqué et **Enregistrer** reste désactivé. Une valeur que le fichier avait
  déjà et que vous n'avez pas touchée est conservée telle quelle.
- **Champs trop longs.** Un champ dont le texte est plus long que
  `limits.max_tag_chars`, ou qui contient plus de valeurs que
  `limits.max_tag_values`, est affiché en lecture seule avec la note « Trop
  long pour être modifié ici ; conservé tel quel dans le fichier ». Il n'est
  jamais réécrit, si bien qu'un enregistrement ne peut pas le couper.
- **Ce qui est conservé.** Tout ce que l'éditeur n'affiche pas (autres clés
  standard, clés personnalisées, images autres que la pochette avant, trames
  binaires) reste dans le fichier avec les mêmes valeurs. L'éditeur indique
  combien de tags de ce genre sont conservés (et « non comptés » quand le
  format contient des trames qui ne peuvent pas être comptées).
  L'enregistrement réencode les éléments que l'éditeur gère, si bien qu'un
  élément conservé peut différer par ses octets (encodage du texte, ordre des
  trames) mais pas par sa valeur.
- **La pochette.** L'éditeur montre la pochette avant, ou la première image du
  fichier quand il n'y a pas de pochette avant, sous forme de vignette.
  - **Changer…** ouvre une boîte de dialogue de fichier pour une image JPEG ou
    PNG (d'au plus `limits.max_cover_bytes`, et elle doit être décodable). Si
    ce n'est pas le cas, l'éditeur en donne la raison et rien ne change.
  - **Retirer** efface la pochette avant. Il est désactivé quand le fichier
    n'a pas de pochette avant : une image affichée uniquement parce qu'il n'y
    a pas de pochette avant n'est là que pour l'affichage et est conservée
    telle quelle.
  - Une pochette qui est dans le fichier mais ne peut pas être affichée (une
    image qui ne se décode pas, ou un GIF, BMP ou WebP) est signalée par
    « Cette pochette ne peut pas être affichée ; elle est conservée telle
    quelle ». **Changer…** et **Retirer** fonctionnent toujours.
  - Le changement est écrit par **Enregistrer** et abandonné par **Annuler**.
    Les pochettes arrière et toutes les autres images ne sont jamais touchées.
    Un format sans place pour les images (WAV avec RIFF INFO, AIFF, ID3v1)
    affiche la zone désactivée. Après un enregistrement, la pochette du
    lecteur montre la nouvelle pochette.
- **Comment fonctionne un enregistrement.** Le fichier est copié à côté de
  l'original, la copie reçoit les tags, est synchronisée et remplace
  l'original, si bien qu'un échec laisse le fichier tel qu'il était. La
  raison s'affiche dans l'éditeur, qui reste ouvert pour réessayer, et dans la
  barre d'état. Seuls les champs que vous avez modifiés sont écrits. Après un
  enregistrement, le tableau montre aussitôt les nouveaux tags, et les
  marqueurs et la forme d'onde sont conservés. Si le fichier n'a pas conservé
  un champ que vous avez modifié, la barre d'état le nomme.
- **Après une mise à jour.** Les pistes d'une version antérieure voient leur
  date, leur genre et d'autres tags remplis discrètement en arrière-plan
  (sans analyse complète).

## Glisser-déposer {#drag-and-drop}

- Faites glisser une piste dans la liste pour la réordonner. Une ligne violette
  montre où elle va atterrir : elle se trouve à la limite de ligne la plus
  proche du pointeur, et seulement dans la liste sous le pointeur. Relâcher
  au-dessus de l'en-tête, d'un bord de colonne, de la barre de défilement ou
  d'une fenêtre qui recouvre la liste (la fenêtre CUE) ne dépose rien.
- Pendant que vous faites glisser une piste, maintenez le pointeur près du
  bord supérieur ou inférieur d'une liste pour la faire défiler : plus vous
  êtes près du bord, plus elle va vite, et elle s'arrête aux extrémités de la
  liste ou quand vous vous éloignez du bord. La molette de la souris fait
  aussi défiler la liste pendant le glissement. La ligne violette continue de
  suivre le pointeur pendant que la liste bouge. Faire glisser des fichiers
  depuis le gestionnaire de fichiers au-dessus d'une liste la fait défiler de
  la même façon là où le système signale la position du pointeur, une fois
  que vous déplacez le pointeur au-dessus de la liste.
- Faites-la glisser sur la liste d'un autre lecteur pour l'y déplacer.
- Faites-la glisser sur un onglet pour l'ajouter à la fin de cette playlist.
- Déposez des fichiers ou des dossiers depuis le gestionnaire de fichiers sur
  une liste pour les insérer à la position du dépôt. Au-dessus de l'en-tête,
  d'un bord de colonne, de la barre de défilement ou d'une fenêtre qui
  recouvre la liste, rien n'est inséré. Si le système ne signale pas la
  position, ils vont à la fin de la liste affichée.

## Pied de page {#footer}

**+ Ajouter** ouvre une boîte de dialogue de fichier, qui démarre dans le
dossier de musique défini dans les Paramètres. **Réinitialiser** (l'icône en
flèche à côté) efface la marque « déjà jouée » grisée de chaque piste de la
playlist, pour chaque lecteur, après avoir demandé « Effacer la marque de
lecture de toutes les pistes de cette playlist ? » (**Annuler**, Échap ou un
clic en dehors conservent les marques). La piste qui est à l'antenne garde son
état et est marquée quand le lecteur la quitte. Le bouton est grisé quand il
n'y a rien à effacer. Le pied de page montre aussi le nombre de pistes, le
temps restant dans la playlist et sa durée totale.

## Fichiers de playlist {#playlist-files}

- **Importer :** Paramètres → Playlists → **Importer M3U / PLS…**, ou déposez
  un fichier `.m3u`, `.m3u8` ou `.pls` sur la fenêtre. Il devient une nouvelle
  playlist nommée d'après le fichier.
  - Les chemins relatifs sont résolus par rapport au dossier du fichier de
    playlist.
  - Les adresses `file://` sont comprises.
  - Les fichiers introuvables sont quand même ajoutés, marqués indisponibles.
  - Les flux Internet sont ignorés ; un message indique combien.
- **Exporter :** le bouton **M3U** de chaque playlist dans les Paramètres
  l'enregistre comme fichier M3U8 avec titres, durées et chemins complets.
