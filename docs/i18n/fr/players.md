# Lecteurs

Chaque colonne est un lecteur. Les lecteurs sont indépendants : chacun a ses
propres onglets de playlist, son transport, son volume et ses sorties.

![Le lecteur 1 à l'antenne : son en-tête avec le gain de loudness, sa pochette, son titre, la piste suivante, le transport, le compte à rebours, le vumètre, le fader et la forme d'onde](../../images/guide/player.png)

## En-tête {#header}

| Élément | Signification |
|---|---|
| `P1` … `Pn` | Numéro du lecteur (la touche numérique qui le lance) |
| Boutons **Masquer** et **Agrandir** / **Restaurer** | Les deux petits boutons à l'extrémité droite de l'en-tête : masquer le lecteur, ou l'agrandir un instant (**Restaurer** tant qu'il est agrandi). Grisés tant que la disposition est verrouillée, **Masquer** sur le dernier panneau affiché et **Agrandir** quand un seul panneau est affiché (voir [Disposition](layout.md#maximize-a-panel)) |
| Point d'état et libellé | **À l'antenne** (rouge), **En pause** (ambre), **Arrêté** (gris) |
| Colonnes étroites | Quand la colonne est trop étroite pour tous les badges, l'en-tête abandonne d'abord les moins importants (le texte d'état, puis **Autres coupées**, l'avis d'entrée, **Stop à la fin**, **Mix** / **Fondu**, le point d'état), si bien que rien ne se chevauche jamais. Le numéro du lecteur, **CUE**, le mode, **BP**, le badge de gain et les boutons **Masquer** et **Agrandir** restent toujours. Élargissez la colonne pour tous les voir |
| Badge **Mix** / **Fondu** | Un fondu enchaîné vers la piste suivante, ou un stop en fondu, est en cours. **Mix** ne s'allume jamais sur un lecteur dont la sortie Main est bit-perfect : il n'enchaîne pas en fondu (voir [Ce que cela coûte](bit-perfect.md#what-it-costs)) |
| Badge **Stop à la fin** | Le lecteur s'arrête quand la piste en cours se termine |
| Badge **Répéter** / **Arrêt après** | La piste en cours se répète, ou arrête le lecteur à sa fin, à cause de sa propre marque dans le menu de la playlist. Survolez pour lire la phrase complète. Le bouton **Stop à la fin** du lecteur l'emporte : tant qu'il est activé, seul son badge s'affiche |
| Badge de gain | Avec la [normalisation du loudness](loudness.md#the-gain-in-the-player-header) activée, le gain avec lequel joue la piste à l'antenne (**+4.0 dB**), ou un **0.0 dB** ambre quand la piste joue telle quelle (pas encore de mesure, une sortie bit-perfect, ou une piste exclue de la normalisation). Survolez pour la raison. Affiché tant que le lecteur joue ou est en pause |
| **BP** / **DSD** | Le badge ne s'affiche que sur un lecteur dont le périphérique Main fonctionne en bit-perfect ; **BP** est allumé tant que la piste en cours parvient intacte à son périphérique Main. **DSD** le remplace pendant qu'une piste DSD sort en DSD, intacte (voir [Sortie bit-perfect](bit-perfect.md)). **Autres coupées** s'affiche à côté quand ce flux DSD tient les autres sources à l'écart de la sortie |
| **SINGLE** \| **CONT** | Mode de lecture (voir plus bas) : un contrôle d'un seul tenant, la moitié allumée est le mode actif |
| **CUE** | Pré-écouter la piste suivante sur la sortie CUE |

Les deux petits boutons à l'extrémité droite de l'en-tête, **Masquer** et
**Agrandir** (ou **Restaurer**), masquent le lecteur ou l'agrandissent un
instant ; les deux mêmes se trouvent dans le menu d'un clic droit sur le fond
de l'en-tête (pas sur l'un de ses boutons) (voir
[Disposition](layout.md#maximize-a-panel)).

## Rangée d'informations {#info-row}

- **Pochette** de la piste à l'antenne, ou un disque vinyle de remplacement.
- **Titre et artiste** de la piste à l'antenne. Un lecteur arrêté montre la
  piste que PLAY va démarrer (sa suivante), avec sa pochette, sa durée et sa
  forme d'onde, prête au cue-in, ou là où vous avez cliqué sur sa forme
  d'onde.
- Une piste lue ou chargée avant la fin de son analyse a déjà sa durée quand
  l'en-tête de son fichier la fournit : le compte à rebours, `écoulé / total`
  et le clic pour se déplacer fonctionnent aussitôt, sur une ligne plate
  jusqu'à ce que la forme d'onde soit prête. Quand l'en-tête ne la stocke pas
  (fichiers AAC bruts, fichiers MP3 sans trame de durée, Matroska et WebM), le
  total affiche « — » et la forme d'onde ne peut pas être cliquée avant la fin
  de l'analyse.
- **Vumètre stéréo** (dans la colonne à droite du lecteur, à côté du fader ;
  il s'étend sur la rangée d'informations et le transport) : le niveau que le
  lecteur émet, après son volume.
  - Une barre continue par canal, sur l'échelle de la norme du type de
    vumètre (le vumètre de crête numérique par défaut : −60 à 0 dBFS, les 20
    dB du haut occupant la moitié de la hauteur). L'échelle est une règle de
    chaque côté des barres, graduée dans les unités propres au vumètre des
    deux côtés. Son haut et son bas (pour le vumètre numérique, le bas de
    l'échelle) sont toujours marqués ; chaque graduation a un trait sur chaque
    règle, et des traits plus courts entre les graduations fonctionnent comme
    ceux d'une règle de mesure : régulièrement espacés sur des valeurs
    rondes, tous les 1 dB sur le vumètre UER et tous les 5 dB sous −20 sur le
    numérique quand le vumètre est assez haut, et tous les 2, 2,5, 5 ou 10 dB
    (ou aucun) là où il est trop court pour eux. Sur le vumètre numérique (et
    le personnalisé), un vumètre haut gradue plus de valeurs : tous les 1 dB
    de −20 à 0, et tous les 5 dB entre les repères de 10 dB sous −20 (−45,
    −55), toujours régulièrement espacés et seulement là où ils tiennent sans
    que les graduations se touchent ; les autres types de vumètre gardent les
    graduations que leur norme donne. Le niveau d'alignement (−18 dBFS sur le
    vumètre numérique) est un trait blanc plus épais sur les deux règles. Rien
    n'est dessiné sur les barres ni entre elles : ce que vous voyez dans les
    barres, c'est uniquement le niveau, le maintien de crête et les couleurs.
  - La barre est verte, jaune à partir du niveau d'alerte (−9 dBFS), et rouge
    à partir du niveau de danger (−3 dBFS). Les autres types de vumètre
    passent au rouge là où leur échelle le fait (à partir de 0 VU, à partir du
    maximum autorisé sur un PPM).
  - Les vumètres K-System montrent deux sections : la barre pleine est le
    niveau moyen (RMS) et la partie plus pâle au-dessus atteint la crête. Leurs
    couleurs sont celles du K-System : vert sous 0, ambre de 0 à +4, rouge
    au-dessus.
  - Les vumètres de crête numérique, K-System et personnalisés gardent le
    niveau le plus élevé allumé un instant sous forme de ligne (le maintien de
    crête ; sa durée est **Maintien de crête** dans
    [Paramètres → Vumètres](settings.md#meters), et 0 le désactive). Les
    vumètres PPM UER, PPM DIN et VU n'ont pas de maintien.
  - Le nombre au-dessus est le niveau le plus élevé depuis le début de
    l'entrée, en dBFS, rouge dans la zone de danger. Il reste après un arrêt
    et repart quand une entrée est lue (la suivante ou la même de nouveau), ou
    quand vous cliquez dessus.
  - Le nombre en dessous est le loudness en LUFS (EBU R128), vert à ±1 LU de
    la cible (−23 LUFS).
  - Le type de vumètre et chaque niveau peuvent être modifiés dans
    [Paramètres → Vumètres](settings.md#meters).
  - **Lectures au-dessus de 0 dBFS.** Le vumètre montre ce que le lecteur
    émet, et cela peut dépasser la pleine échelle. La barre s'arrête en haut de
    l'échelle, si bien que le même rouge montre 0 dBFS et tout ce qui le
    dépasse ; seul le nombre au-dessus dit de combien, avec son signe (par
    exemple `+3.5`).
    - Un fichier peut lui-même porter des niveaux au-dessus de la pleine
      échelle (un fichier en virgule flottante, ou un fichier avec perte dont
      les crêtes décodées la dépassent).
    - Convertir la fréquence peut créer des crêtes entre les échantillons : un
      signal qui touche 0 dBFS lit environ +3 dBFS après 44,1 → 48 kHz.
      L'option true peak lit aussi ces crêtes.
    - Le vumètre lit chaque lecteur isolément, pas la somme sur le
      périphérique : deux lecteurs sur une même sortie peuvent s'additionner
      au-delà de la pleine échelle sans que l'un ou l'autre vumètre ne le
      montre.
    - Rien dans le lecteur n'ajoute de gain au-dessus de 100 %. Un périphérique
      entier écrête à la pleine échelle ; un périphérique en virgule flottante
      reçoit le niveau tel quel et le système audio ou le pilote écrête.
- **Fader de volume** (à droite du vumètre, de la même hauteur). Il suit une
  loi de table de mixage : il est fin près du haut et plus raide vers le bas,
  si bien que le baisser un peu baisse un peu le niveau (90 % de la course
  font environ −2,5 dB, 50 % environ −15,9 dB, 10 % environ −48 dB). Le haut
  est 0 dB et le bas est le silence. Des repères en travers du fader marquent
  0, −2, −4, −6, −8, −10, −12, −15, −20, −25, −30, −40, −50 et −∞ dB, avec le
  niveau écrit à leur droite.
  - Faites-le glisser : le fader se déplace d'autant que le pointeur, à partir
    d'où il se trouve, si bien que l'attraper ne le fait jamais sauter ; un
    clic sans déplacement ne change rien. Tirer au-delà du haut ou du bas puis
    revenir laisse le bouton à l'extrémité jusqu'à ce que le pointeur revienne
    à l'endroit qu'il avait atteint, si bien que le bouton ne se retrouve
    jamais loin du pointeur.
  - Utilisez la molette de la souris : un pas par cran (environ 0,6 dB près de
    0 dB).
  - Maintenez **Ctrl** ou **Shift** pendant le glissement ou la molette pour
    le déplacer dix fois plus finement.
  - Double-cliquez sur le bouton pour revenir à 0 dB (un double clic ailleurs
    sur le fader ne fait rien).

  Le niveau en dB (une décimale, ou −∞ pour le silence) est écrit sous le
  fader, sur la même rangée que la ligne de loudness du vumètre, et l'infobulle
  l'indique aussi. Une session enregistrée avant cette loi conserve le niveau
  de chaque lecteur ; seule la position du fader change.
  Sur un lecteur dont la sortie Main est bit-perfect, le fader est verrouillé
  à 100 % et grisé, il affiche un 0.0 atténué, et son infobulle en donne la
  raison (voir [Ce que cela coûte](bit-perfect.md#what-it-costs)).
- **Titre, artiste** et la ligne de la **suivante**, avec un carré vert.
  Quand le CUE est activé, la position de pré-écoute s'affiche en bleu.

## Transport {#transport}

| Bouton | Action |
|---|---|
| **PLAY / SUIV.** (grand) | À l'arrêt : démarrer la piste suivante. À l'antenne : fondre dans la piste suivante (la durée du fondu se règle dans les [Paramètres](settings.md)) ; pendant le fondu d'un enchaînement lancé par Suivante ou par un segue, une bande verte se vide à travers le bouton sur la durée du fondu, son bord allant de gauche à droite comme celui d'une cartouche (un enchaînement lancé par Précédente se vide plutôt sur le bouton Précédente). En pause : reprendre. Quand la piste à l'antenne est aussi la suivante, PLAY redémarre cette piste avec le fondu habituel. Sur une sortie bit-perfect, il bascule aussitôt, sans fondu, et son infobulle le dit (voir [Ce que cela coûte](bit-perfect.md#what-it-costs)). |
| **Stop** | Arrêter aussitôt (avec une courte rampe anti-clic) |
| **Stop en fondu** | Fondu sortant puis arrêt. Pendant qu'un stop en fondu se déroule, une bande se vide à travers le bouton sur la durée du fondu, son bord allant de gauche à droite |
| **Pause** | Mettre en pause ou reprendre ; clignote en ambre pendant la pause |
| **Stop à la fin** (un triangle de lecture puis un carré) | S'arrêter quand la piste en cours se termine, une seule fois. En mode SINGLE, il n'est disponible que pendant que la piste en cours se répète : il met fin à la répétition quand le passage en cours se termine. Pour s'arrêter après une piste à chaque fois qu'elle est lue, ou pour répéter une piste, utilisez son menu dans la playlist (voir [Playlists](playlists.md)) |
| **Précédente** (une barre et deux triangles) | À l'antenne : revenir en fondu vers la piste que ce lecteur a jouée avant, comme le fait Suivante. Appuyez de nouveau pour continuer à reculer. La piste que vous quittez devient la suivante. Sur une sortie bit-perfect, il bascule aussitôt, sans fondu (voir [Ce que cela coûte](bit-perfect.md#what-it-costs)). |
| **Reprendre au début** (une barre et un triangle) | Retour au début de la piste en cours (son cue-in). Un lecteur en pause reste en pause. |

Au clavier, `Up` et `Down` déplacent la sélection dans la playlist, `Enter`
définit la piste sélectionnée comme suivante, et un second `Enter` sur elle
dans le délai d'un double clic la lit. Voir [Clavier](keyboard.md).

Les boutons qui ne peuvent pas agir pour l'instant sont grisés : Stop et
Reprendre au début quand rien n'est chargé, Pause et Stop en fondu à l'arrêt,
Précédente sans piste antérieure ou pendant un fondu. Un lecteur se souvient
des 50 dernières pistes qu'il a jouées (`players.history_len` dans
`config.json`, de 0 à 1000).

## Modes {#modes}

- **CONT (continu) :** au point MIX, le lecteur démarre la piste suivante et
  la fait chevaucher la fin de la piste en cours. Voir
  [Marqueurs et mixage](markers-and-mixing.md).
- **SINGLE :** chaque piste s'arrête à sa fin. *Stop à la fin* n'est pas
  disponible dans ce mode, parce que chaque piste s'arrête déjà, sauf pendant
  que la piste en cours se répète : il met alors fin à la répétition quand le
  passage en cours se termine.

## Compte à rebours {#countdown}

Le grand nombre est le temps restant jusqu'à la fin de la piste (son
cue-out), avec les dixièmes. Le temps écoulé et le total se trouvent sur la
rangée sous la forme d'onde, à droite. Pendant les dernières secondes avant
la fin (10 par défaut, réglable dans les Paramètres), le compte à rebours
clignote en rouge.

## Forme d'onde {#waveform}

- La partie déjà jouée est dessinée dans la couleur de la forme d'onde ; le
  reste est plus pâle. Avec une couleur de forme d'onde violette, la ligne MIX
  violette ressort moins ; choisissez une autre couleur de forme d'onde si
  vous devez les distinguer.
- Le contour montre les crêtes, en filigrane ; le corps plein à l'intérieur
  est le niveau moyen (RMS). Sur une piste forte, les crêtes remplissent la
  hauteur, et le corps montre encore où la piste est plus faible ou plus
  forte.
- Une zone ombrée bleue au début marque l'**intro**, et un badge la décompte.
  L'intro ne s'affiche que lorsqu'elle a été définie.
- Une zone ombrée orange à la fin marque l'**outro**, avec son propre compte à
  rebours.
- Une ligne violette en tirets (la couleur du badge **Mix**, pour qu'on ne la
  confonde pas avec l'outro orange) avec une étiquette **MIX** marque l'endroit
  où la piste suivante démarre en mode continu. Elle est atténuée en mode
  single, et sur un lecteur dont la sortie Main est bit-perfect, qui n'enchaîne
  pas (voir [Ce que cela coûte](bit-perfect.md#what-it-costs)).
- Le fichier entier est dessiné. Le début et la fin silencieux que la lecture
  ignore (avant le cue-in et après le cue-out) sont dessinés plus sombres,
  avec un trait fin là où la lecture commence et finit. Avec **Utiliser le
  cue-in et le cue-out** désactivé (Paramètres → Lecteurs), rien n'est plus
  sombre et les deux lignes sont atténuées : la lecture va du début à la fin
  du fichier, et le cue-in et le cue-out de ce guide désignent ces deux
  extrémités.
- Survolez pour voir le temps sous le pointeur. **Cliquez pour vous y
  déplacer.** Sur un lecteur arrêté, un clic choisit l'endroit où **PLAY**
  démarre la piste suivante : la tête de lecture et le compte à rebours s'y
  déplacent, et rien ne joue tant que vous n'appuyez pas sur PLAY. Choisir une
  autre piste suivante, la déplacer ou la retirer, ou Stop, revient au
  cue-in ; de même que toute autre façon de démarrer une piste, et Reprendre
  au début, Précédente et l'avance automatique utilisent toujours le cue-in.
  Un clic avant le cue-in (dans le début plus sombre) choisit le cue-in. Un
  clic au cue-out ou après (dans la fin plus sombre) annule un choix
  antérieur : PLAY démarre au cue-in. Un clic est un appui et un relâchement
  sans que le pointeur bouge de plus de quelques pixels.
- **Appuyer et glisser** déplace la vue zoomée le long de la piste, comme si
  vous l'agrippiez. Un glissement ne provoque jamais de saut, et sans zoom il
  ne fait rien. Alt + glissement modifie toujours les marqueurs.
- **Molette de la souris** sur la forme d'onde : zoomer et dézoomer autour du
  pointeur, jusqu'au détail le plus fin dont dispose l'analyse.
  **Shift+molette** (ou une molette latérale) déplace le long de la piste.
  Pendant le zoom, la vue suit la position de lecture, sauf pendant 10
  secondes après que vous avez zoomé ou déplacé (`ui.follow_current_grace_secs` ;
  0 désactive le suivi). **Vue complète**, dans le coin supérieur droit,
  dézoomer complètement ou une nouvelle piste montrent de nouveau toute la
  piste.

## CUE (pré-écoute) {#cue-pre-listen}

Appuyer sur **CUE** (ou sur **Pré-écouter sur le CUE** dans le menu d'une
piste) joue la piste sur la sortie CUE du lecteur, par exemple un casque, et
ouvre une petite **fenêtre CUE** pour ce lecteur. Plusieurs fenêtres peuvent
être ouvertes, une par lecteur. Voir les [Paramètres](settings.md) pour
choisir le périphérique CUE. Un lecteur a besoin d'une sortie Cue : sans elle,
**CUE** et **Pré-écouter sur le CUE** sont grisés, et les survoler l'indique.

Avec une seule carte son, la sortie Cue peut être la sortie d'antenne
elle-même (ou la partager avec un autre lecteur ou le mur de cartouches). Une
pré-écoute y est entendue à l'antenne, donc :

- tant que rien ne joue sur cette sortie, le CUE démarre aussitôt ;
- tant que quelque chose y joue (un lecteur en lecture ou en pause, une
  cartouche, une autre pré-écoute, la pré-écoute de la bibliothèque), le CUE
  demande d'abord : **Pré-écouter quand même** ou **Annuler** (Annuler est le
  choix par défaut : Échap et Entrée annulent). La même question vient du menu
  de la piste, de la fenêtre CUE, du mur de cartouches et de la fenêtre de la
  bibliothèque ;
- quand quelque chose démarre sur cette sortie pendant qu'y joue une
  pré-écoute que vous n'avez pas confirmée, la pré-écoute s'arrête et la
  barre d'état en donne la raison. Une pré-écoute que vous avez confirmée
  continue de jouer.

L'infobulle du bouton CUE indique quand sa pré-écoute sort sur la sortie
d'antenne.

![La fenêtre CUE du lecteur 4, pré-écoutant sa piste suivante](../../images/guide/cue-window.png)

La fenêtre montre :

- le titre et l'artiste ;
- la forme d'onde du fichier entier avec la position du CUE. Elle fonctionne
  comme celle du lecteur : cliquez pour vous y déplacer, zoomez avec la
  molette, glissez pour avancer, **Vue complète**, les comptes à rebours de
  l'intro et de l'outro, et les marqueurs d'intro, d'outro et MIX, que vous
  modifiez ici comme sur le lecteur (voir
  [Marqueurs et mixage](markers-and-mixing.md)). Un CUE joue le fichier
  entier, donc rien n'est dessiné plus sombre, le cue-in et le cue-out sont
  des lignes atténuées, et l'outro décompte jusqu'à la fin du fichier. Son
  zoom lui est propre : la forme d'onde du lecteur ne bouge pas. Pendant que
  le CUE joue, une vue zoomée suit sa position comme celle du lecteur ; un
  CUE en pause garde la vue que vous avez définie, ce qui vous permet de
  zoomer et de placer des marqueurs. Un CUE lancé après la fermeture de sa
  fenêtre, ou sur une autre piste, montre le fichier entier ;
- le temps écoulé et le temps restant jusqu'à la fin du fichier (un CUE joue
  des fichiers entiers) ;
- **Mettre le CUE en pause** / **Reprendre le CUE**, **Arrêter le CUE** et
  **Définir comme suivante**. **Définir comme suivante** fait de la piste
  pré-écoutée la suivante du lecteur et laisse le CUE se poursuivre. Il est
  grisé quand la piste est déjà la suivante. Si la piste pré-écoutée est celle
  à l'antenne, elle est rejouée quand le passage en cours se termine.

Pendant que le CUE est en pause, son bouton **Mettre le CUE en pause**
(affiché comme **Reprendre le CUE**) clignote en ambre, comme celui du lecteur.

Un déplacement sur un CUE en pause le laisse en pause. Le bouton de fermeture
de la fenêtre, ou **Arrêter le CUE**, arrête le CUE.

Pendant qu'un CUE tourne, définir une suivante (double-clic) ou un simple
clic sur une ligne déplace le CUE sur cette piste, depuis son cue-in ; s'il
était en pause, il rejoue. Une piste dont le fichier est introuvable ou
illisible laisse le CUE là où il est.
