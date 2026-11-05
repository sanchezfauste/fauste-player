# Bibliothèque

La bibliothèque est l'ensemble des pistes que Fauste Player connaît : chaque
fichier que vous avez ajouté à une playlist, à une cartouche ou à la
bibliothèque elle-même est une piste, avec ses tags, ses marqueurs et ses
réglages de loudness. La fenêtre de la bibliothèque les liste, les regroupe en
*catégories*, les trouve par texte et les envoie vers les playlists et le mur
de cartouches. C'est une fenêtre sur l'ordinateur et sur une tablette ; un
téléphone montre la bibliothèque dans son onglet **Bibliothèque** à la place
(voir [Sur un téléphone](#on-a-phone)).

## Ouvrir la fenêtre {#opening-the-window}

Cliquez sur la tuile **Bibliothèque** dans la barre supérieure. Elle est
allumée tant que la fenêtre est ouverte ; cliquez de nouveau dessus, ou sur le
bouton de fermeture de la fenêtre, pour la fermer. La fenêtre flotte
au-dessus des lecteurs et du mur de cartouches et ne les bloque pas : rien
dedans n'arrête l'audio ni les raccourcis de la fenêtre principale.

Déplacez-la par sa barre de titre et redimensionnez-la par ses bords et ses
coins. La barre de titre a trois boutons : **Réduire** (il ne reste que la
barre de titre, là où se trouvait la fenêtre, ou en haut de la zone principale
quand la fenêtre est agrandie ; cliquez de nouveau pour la déplier),
**Agrandir** (la fenêtre remplit la zone principale ; **Restaurer** la
ramène) et **Fermer**. Une fenêtre qui ne tiendrait plus, après que la fenêtre
principale a rétréci ou est passée sur un écran plus petit, est ramenée à
l'intérieur de la zone principale.

La position et la taille de la fenêtre, son état réduit ou agrandi, la largeur
de l'arborescence, les colonnes, l'ordre de tri et la catégorie sélectionnée
sont enregistrés avec la session et reviennent au démarrage suivant. Le fait
que la fenêtre soit ouverte est enregistré aussi, mais rien ne joue de
lui-même à cause de cela. Le texte du champ de recherche n'est pas enregistré.

## Catégories {#categories}

L'arborescence à gauche regroupe les pistes. En haut, **Tous les titres** ;
puis vos catégories, qui peuvent contenir des sous-catégories ; à la fin,
**Sans catégorie**, les pistes qui n'appartiennent à aucune catégorie. Chaque
élément montre sa couleur et un compte : une catégorie compte ses propres
pistes et celles de ses sous-catégories. Une piste appartient à une catégorie
au plus.

- **Cliquez** sur un élément pour lister ses pistes dans le tableau. Une
  catégorie liste aussi les pistes de ses sous-catégories.
- La **flèche** avant une catégorie ouvre ou ferme ses sous-catégories. Celles
  qui sont ouvertes sont enregistrées.
- **Nouvelle catégorie** (＋ au-dessus de l'arborescence) demande un nom, une
  couleur et un *type*. Une nouvelle catégorie prend sa couleur et son type de
  la catégorie dans laquelle elle est créée ; le type est ce que prend une
  piste quand elle rejoint la catégorie, et le changer ensuite ne modifie pas
  les pistes qui s'y trouvent déjà. Un nom ne peut pas être vide ni répéter
  le nom d'une catégorie sœur ; les catégories ne s'imbriquent que jusqu'à une
  limite.
- **Faites un clic droit** sur une catégorie (ou un appui long sur un écran
  tactile) pour **Nouvelle sous-catégorie…**, **Renommer…**, **Couleur**,
  **Type** et **Supprimer…**. Sur l'ordinateur, le menu a aussi **Surveiller un
  dossier…** (**Changer le dossier surveillé…** et **Arrêter de surveiller le
  dossier** une fois qu'elle en surveille un) et **Retirer les pistes
  manquantes…** ; voir [Dossiers surveillés](#watched-folders) et
  [Retirer des pistes](#removing-tracks).
- **Faites glisser** une catégorie pour la déplacer : sur une autre catégorie
  pour l'y imbriquer, ou dans l'espace entre deux catégories pour la placer là.
  Une catégorie ne peut pas aller dans elle-même. Sur un écran tactile,
  l'arborescence ne prend que des appuis et défile sous un doigt ; une
  catégorie ne peut pas y être glissée.

**Supprimer…** nomme ce qui part avec la catégorie : ses sous-catégories et
chaque piste qu'elles contiennent sont retirées de la bibliothèque. Les
fichiers ne sont jamais supprimés. Si l'une de ces pistes est utilisée, une
seconde question dit combien d'entrées de playlist et de cartouches les
utilisent : **Retirer quand même** retire les entrées de leurs playlists et
vide les cartouches. Rien n'est retiré tant que l'une d'elles est à
l'antenne ; un avis dit ce qui joue.

La même arborescence se trouve dans **Paramètres → Bibliothèque**, pour créer,
renommer, recolorer, changer le type, déplacer et supprimer des catégories
sans ouvrir la fenêtre (elle n'a ni *Tous les titres*, ni *Sans catégorie*,
ni sélection).

## Dossiers surveillés {#watched-folders}

Sur l'ordinateur, une catégorie peut surveiller un dossier. Faites un clic
droit dessus et choisissez **Surveiller un dossier…**, puis choisissez le
dossier. La catégorie montre une marque ⟳ avant son compte ; survolez-la pour
lire le dossier.

- Chaque fichier audio du dossier et de ses sous-dossiers rejoint la catégorie.
  Les sous-dossiers ne créent pas de sous-catégories.
- Une piste qui est déjà dans une autre catégorie y reste. Une piste qui n'est
  dans aucune catégorie rejoint celle-ci. Définir, changer ou arrêter une
  surveillance ne déplace jamais une piste : les pistes d'un dossier qui n'est
  plus surveillé restent dans leur catégorie.
- Pendant que l'application tourne, le dossier est suivi : un nouveau fichier
  rejoint la catégorie une fois qu'il est resté tranquille un instant
  (`library.watch_debounce_ms`, 1 seconde par défaut, pour qu'un fichier encore
  en cours de copie ne soit pas lu à moitié écrit). Le dossier est aussi
  analysé à chaque démarrage de l'application, pour rattraper ce qui a changé
  pendant qu'elle était fermée.
- Un fichier qui disparaît du dossier (supprimé, déplacé ou renommé) laisse sa
  piste dans la bibliothèque, marquée comme manquante, avec ses marqueurs. Si
  le fichier revient, la piste redevient normale d'elle-même. Un fichier
  renommé rejoint comme une nouvelle piste sous son nouveau nom.
- Les dossiers surveillés peuvent être imbriqués. Un fichier sous deux d'entre
  eux va dans la catégorie du plus interne ; arrêter l'un d'eux laisse l'autre
  suivi. Un même dossier ne peut pas être surveillé par deux catégories :
  l'application dit quelle catégorie l'a déjà.
- Rien de ce que fait un dossier surveillé ne charge ni ne joue une piste.

Si le dossier ne peut pas être suivi, la marque ⟳ change de couleur et son
infobulle dit pourquoi : *Dossier surveillé introuvable : …. Il est recherché
de nouveau.* (par exemple un disque qui n'est pas monté), *Dossier surveillé
illisible : …. Une nouvelle tentative aura lieu.*, ou *Les changements dans …
ne sont pas suivis ; le dossier est analysé au démarrage et une nouvelle
tentative aura lieu.* (le système a refusé de le surveiller). Un dossier
enregistré sous la forme d'un chemin qui n'est pas complet (modifié à la main)
n'est pas surveillé du tout, et la marque le dit.

Toutes les 30 secondes (`tuning.missing_recheck_ms`), l'application vérifie
que chaque dossier surveillé est toujours là, si bien qu'un disque débranché
pendant qu'elle tourne est signalé aussi. Un dossier en difficulté est
réessayé à ce rythme et analysé dès qu'il est de retour. Un dossier qui n'est
pas là, ne peut pas être lu, ou est vide alors que la bibliothèque a encore
des pistes qui en viennent (un disque ou un partage non monté ressemble
souvent à un dossier vide) compte comme introuvable. La surveillance ne marque
alors aucune de ses pistes comme manquante et ne retire rien, y compris quand
le dossier se trouve dans un autre dossier surveillé.

Si vous videz exprès un dossier surveillé, l'avertissement reste jusqu'à ce
que des fichiers y reviennent ou que vous arrêtiez de le surveiller, et ses
pistes restent dans la bibliothèque sans être marquées comme manquantes. Pour
les effacer, sélectionnez-les dans le tableau et choisissez **Retirer de la
bibliothèque…**, ou choisissez **Réanalyser** sur elles : l'analyse marque
comme manquantes les pistes dont le fichier a disparu, et **Retirer les pistes
manquantes…** les retire ensuite (si le dossier est toujours surveillé, son
avertissement disparaît à la vérification suivante).

Une exception à la règle ci-dessus : une piste qui n'a jamais été analysée est
quand même marquée comme manquante quand l'analyse ne trouve pas son fichier,
où qu'il soit.

Un sous-dossier qui ne peut pas être lu est ignoré et le journal le dit ; les
liens de dossier (liens symboliques vers des dossiers) ne sont pas suivis.

Les nouveaux fichiers passent à l'analyse derrière tout ce dont un lecteur ou
une cartouche a besoin, si bien qu'un dossier de milliers de fichiers prend un
moment ; la progression de l'analyse le montre.

Le même menu et la même marque se trouvent dans **Paramètres → Bibliothèque**.
Ils ne sont pas sur Android, qui ne surveille pas de dossiers : là, une
catégorie peut prendre ses fichiers dans un dossier de l'appareil que vous
relisez quand vous le choisissez (voir [Android →
Bibliothèque](android.md#library)).

## Le tableau des pistes {#the-track-table}

Le tableau liste les pistes de l'élément sélectionné, dans les mêmes colonnes
que les tableaux de playlist (celles choisies dans **Paramètres → Playlists →
Colonnes du tableau**, sans la position `#`) suivies de **Catégorie** et
**Type**. Une cellule étroite à gauche signale une piste dont le fichier est
manquant ou ne peut pas être lu. Une piste qui attend son analyse est signalée
à droite de sa cellule **Titre** (donc seulement quand la colonne Titre est
affichée).

- Faites glisser un séparateur d'en-tête pour changer la largeur d'une
  colonne ; cliquez sur un en-tête pour trier selon lui et de nouveau pour
  inverser. Les largeurs et l'ordre de tri sont enregistrés.
- **Cliquer** sélectionne une ligne, **Ctrl** (**Cmd** sous macOS) **clic**
  en ajoute ou en retire une, **Shift clic** sélectionne les lignes entre le
  dernier clic et celui-ci. La sélection ne garde que des lignes qui sont
  listées : une ligne qui quitte le tableau, à cause d'une recherche, d'une
  autre catégorie ou d'un retrait, quitte la sélection. Chaque action
  s'applique aux lignes sélectionnées dans l'ordre du tableau.
- **Ajouter des fichiers** (au-dessus du tableau) choisit des fichiers audio et
  les ajoute à la bibliothèque, dans la catégorie sélectionnée (sans catégorie
  quand *Tous les titres* ou *Sans catégorie* est sélectionné).

Sur un écran tactile, les lignes ne prennent que des appuis et défilent sous un
doigt ; un appui long ouvre le menu, et les pistes sont envoyées avec les menus
(plus bas), non en les faisant glisser.

## Recherche {#search}

Le champ au-dessus du tableau trouve des pistes par texte. Il cherche dans le
titre, l'artiste, l'album, l'artiste de l'album, le compositeur, le genre, le
commentaire et le nom de fichier, sans tenir compte de la casse ni des accents,
et une piste est listée quand elle contient chaque mot que vous avez saisi (dans
n'importe quel ordre et dans n'importe lequel de ces champs). Il cherche dans
l'élément sélectionné ; activez **Toute la bibliothèque** pour chercher dans
toutes les pistes quelle que soit la sélection. La portée est enregistrée ; le
texte ne l'est pas. Effacez le champ pour revoir l'élément entier.

## Le menu d'une ligne {#the-row-menu}

Faites un clic droit sur une ligne. Si la ligne n'est pas sélectionnée, elle
devient la sélection ; sinon le menu agit sur chaque ligne sélectionnée. La
première ligne montre le titre de la piste quand le menu concerne une piste, et
combien de pistes sont sélectionnées quand il en concerne plusieurs.

| Élément | Action |
|---|---|
| Ajouter à ▸ | Une playlist : les pistes sont ajoutées à sa fin |
| Ajouter à une cartouche ▸ | Une cartouche de la page affichée du mur de cartouches : les pistes remplissent la cartouche choisie et les cartouches vides suivantes de la page, dans l'ordre de la grille. Les pistes qui n'y tiennent pas sont laissées de côté et un avis les compte |
| Déplacer vers la catégorie ▸ | Une catégorie, ou Sans catégorie |
| Type ▸ | Musique, Jingle, Effet, Publicité ou Voix |
| Modifier les tags… | L'éditeur de tags (une piste) |
| Loudness… | La fenêtre de loudness (une piste) |
| Réanalyser | Analyser de nouveau les pistes |
| Rétablir les marqueurs automatiques | Abandonner les marqueurs manuels |
| Afficher l’emplacement du fichier | Ouvrir le dossier du fichier (une piste) |
| Retirer de la bibliothèque… | Retirer les pistes de la bibliothèque, après confirmation (plus bas) |

## Pré-écoute {#pre-listening}

La barre sous la barre de titre pré-écoute la piste sélectionnée sur la propre
sortie **Bibliothèque** de la bibliothèque (choisie dans [Paramètres → Sorties
audio](settings.md#audio-outputs)), par exemple un casque. Un téléphone
pré-écoute plutôt depuis sa liste de pistes (voir [Sur un
téléphone](#on-a-phone)).

La barre montre :

- **Pré-écouter**, **Mettre la pré-écoute en pause** et **Arrêter la
  pré-écoute** ;
- le grand temps restant de la piste, son titre, son artiste et sa catégorie ;
- un petit vumètre à droite. Il ne lit que cette pré-écoute, jamais le vumètre
  d'un lecteur ou du mur de cartouches ; cliquez dessus pour redémarrer son
  maximum ;
- la forme d'onde du fichier entier, avec ses marqueurs, et une ligne
  `écoulé / total` dessous.

**Pré-écouter** démarre la piste sélectionnée à son cue-in et la joue jusqu'à
la fin du fichier. **Pré-écouter** sur une pré-écoute en pause la reprend, et
sur une pré-écoute en cours ne fait rien : Arrêter la pré-écoute, puis
**Pré-écouter**, repart du cue-in. La normalisation du loudness s'applique
comme sur le CUE d'un lecteur.

La forme d'onde fonctionne comme dans la fenêtre CUE d'un lecteur : cliquez
dessus pour vous déplacer (un clic ou le relâchement de la souris déplace ; un
glissement non), zoomez et déplacez-la avec la molette, et modifiez les
marqueurs et ouvrez **Loudness…** depuis son menu (voir [Modifier les
marqueurs](markers-and-mixing.md#editing-markers)). Tant que la pré-écoute est
arrêtée, un clic définit plutôt l'endroit où **Pré-écouter** démarre ensuite.

**La pré-écoute suit la sélection.** Avec plusieurs lignes sélectionnées, c'est
la dernière sur laquelle vous avez cliqué qu'elle utilise.

- Pendant qu'elle joue, sélectionner une autre ligne passe à cette piste depuis
  son cue-in et continue de jouer, sans rien demander.
- Pendant qu'elle est en pause, la nouvelle piste est chargée en pause.
- Pendant qu'elle est arrêtée, elle reste arrêtée et la barre montre la
  nouvelle piste.
- Ne rien sélectionner (aucune ligne sélectionnée, ou la ligne sélectionnée
  filtrée) laisse une pré-écoute en cours telle quelle.
- Sélectionner une piste qui ne peut pas être lue (son fichier est manquant ou
  illisible) arrête la pré-écoute, et la barre dit que le fichier ne peut pas
  être lu.

**Ce qui l'arrête.** Arrêter la pré-écoute ; retirer sa piste de la
bibliothèque ; fermer la fenêtre (la réduire ne l'arrête pas). Elle ne joue
jamais après un redémarrage.

**Sans sortie Bibliothèque** (sur l'ordinateur ; Android joue sur la sortie par
défaut à la place, voir [Android → Bibliothèque](android.md#library)),
**Pré-écouter** est grisé (survolez-le pour la raison) et une ligne à côté dit
que la pré-écoute nécessite une sortie CUE de la bibliothèque, avec un lien qui
ouvre Paramètres → Sorties audio sur la ligne **Bibliothèque**. La barre montre
toujours la piste, sa forme d'onde et ses marqueurs.

**Sur une sortie d'antenne.** La sortie Bibliothèque peut être une sortie
d'antenne (le même périphérique et les mêmes canaux que le Main d'un lecteur ou
du mur de cartouches). Elle fonctionne alors comme le CUE des lecteurs et du
mur de cartouches (voir [CUE (pré-écoute)](players.md#cue-pre-listen)) : tant
que quelque chose y joue, **Pré-écouter** demande d'abord (**Pré-écouter quand
même** ou **Annuler**), et une pré-écoute que vous n'avez pas confirmée
s'arrête quand quelque chose démarre sur cette sortie.

## Glisser vers les playlists et les cartouches {#dragging-to-playlists-and-carts}

Faites glisser avec la souris les lignes sélectionnées (ou la ligne d'où vous
commencez à glisser, quand elle n'est pas sélectionnée) :

- sur un **onglet de playlist** d'un lecteur : les pistes vont à la fin de
  cette playlist ;
- sur un **tableau de playlist** : les pistes vont là où la ligne violette le
  montre ;
- sur une **cartouche** : la cartouche reçoit la première piste (une cartouche
  contient un seul fichier), en remplaçant son fichier quand elle est arrêtée.
  Une cartouche en lecture ou sur le CUE refuse le dépôt, parce qu'un nouveau
  fichier l'arrêterait à l'antenne.

## Retirer des pistes {#removing-tracks}

**Retirer de la bibliothèque…** demande d'abord : les fichiers ne sont pas
supprimés. Quand l'une des pistes est utilisée, une seconde question compte les
entrées de playlist et les cartouches qui les perdent. Une piste qui est à
l'antenne (en lecture dans un lecteur ou sur le mur de cartouches) ne peut pas
être retirée : rien n'est retiré et un avis dit ce qui joue.

**Retirer les pistes manquantes…** (dans le menu de la catégorie, sur
n'importe quelle catégorie) retire les pistes de cette catégorie et de ses
sous-catégories dont le fichier est manquant et qu'aucune entrée de playlist ni
cartouche n'utilise. Il demande d'abord, avec leur nombre ; aucun fichier n'est
touché. Les pistes utilisées sont conservées, et une catégorie qui n'en a aucune
le dit dans un avis. Utilisez-le pour faire le ménage après que des fichiers
ont quitté un dossier surveillé : l'application ne les retire jamais d'elle-même.

## Depuis une playlist {#from-a-playlist}

Sur l'ordinateur, le menu de piste d'une ligne de playlist a **Afficher dans la
bibliothèque** et **Déplacer vers la catégorie ▸**, après **Loudness…**.
*Afficher dans la bibliothèque* ouvre la fenêtre sur la catégorie de la piste
et sélectionne sa ligne ; *Déplacer vers la catégorie* classe la piste sans
ouvrir la fenêtre. Pour une piste qui n'est pas dans la bibliothèque, *Afficher
dans la bibliothèque* est affiché grisé et *Déplacer vers la catégorie* n'est
pas affiché.

## Sur un téléphone {#on-a-phone}

Un téléphone n'a pas de fenêtre de bibliothèque : son onglet **Bibliothèque**
s'ouvre sur les catégories (*Tous les titres*, vos catégories, *Sans catégorie*
et **Musique de l'appareil**), et un appui sur l'une liste ses pistes avec une
recherche et **Toute la bibliothèque**. Cochez des pistes pour les ajouter à une
playlist, appuyez longuement sur une ligne pour son menu, pré-écoutez une piste
avec son 🎧, et ajoutez la musique de l'appareil à une catégorie avec le **＋**
de sa liste. Sur Android, sans sortie Bibliothèque, la pré-écoute joue sur la
sortie par défaut. Voir [Android → Bibliothèque](android.md#library).
