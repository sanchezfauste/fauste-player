# Mur de cartouches

Le mur de cartouches est le panneau de boutons sous les lecteurs, ou à côté
d'eux (voir [Disposition](layout.md#move-and-resize-the-cartwall)). Chaque
bouton, une **cartouche**, joue un son instantanément : jingles, effets,
spots. Les cartouches jouent sur leurs propres sorties, indépendamment des
lecteurs.

![Le mur de cartouches avec une cartouche en lecture](../../images/guide/cartwall.png)

## Utilisation {#using-it}

- **Cliquez sur une cartouche** pour la lancer. **Cliquez de nouveau dessus**
  pour l'arrêter.
- **Faites glisser des pistes depuis la [fenêtre de la bibliothèque](library.md)**
  sur une cartouche pour y mettre la première (elle remplace le fichier quand
  la cartouche est arrêtée). Une cartouche en lecture ou sur le CUE refuse le
  dépôt.
- **Tout arrêter** (à l'extrémité droite de la barre) arrête toutes les
  cartouches en lecture, sur toutes les pages. Son libellé indique combien
  jouent, comme dans **Tout arrêter (2)** ; quand aucune ne joue, il est
  grisé et n'affiche aucun nombre.
- Pendant qu'une cartouche joue, sa bordure et une barre qui grandit depuis
  la gauche au fil de la lecture prennent la [couleur](#setting-up-carts) de
  la cartouche (rouge pour une cartouche sans couleur), un fin trait marque le
  bord lu sur la forme d'onde, et son temps décompte. Le trait se déplace
  de façon fluide, même sur une cartouche courte.
- Une fois le fichier d'une cartouche analysé, sa forme d'onde apparaît en
  transparence derrière son nom, ce qui permet de distinguer un son qui
  attaque aussitôt d'un son qui monte en fondu. Elle couvre ce que la
  cartouche joue, de son cue-in à son cue-out.
- Les cartouches **se superposent** par défaut : en lancer une deuxième
  n'arrête pas la première. Une cartouche réglée sur **Arrêter les autres
  cartouches au lancement** arrête d'abord toutes les autres cartouches à
  l'antenne, sur n'importe quelle page.
- Une cartouche réglée sur **En boucle** repart de son cue-in lorsqu'elle
  atteint sa fin, sans interruption, jusqu'à ce que vous l'arrêtiez.
- Un **clic droit** sur une cartouche donne plus d'options :

| Élément | Action |
|---|---|
| Pré-écouter sur le CUE | La jouer sur la sortie CUE du mur de cartouches (grisé quand le mur de cartouches n'a pas de sortie Cue). Sur la sortie antenne, elle demande d'abord tant que quelque chose y joue (voir [CUE](players.md#cue-pre-listen)) |
| Arrêter | L'arrêter |
| Modifier… | L'ouvrir dans les Paramètres |

Le menu d'une cartouche vide ne contient que **Modifier…**, pour choisir son
fichier.

- **Pages :** les onglets à côté de **CARTOUCHES** changent de page. Un point
  rouge indique qu'une cartouche de cette page est en lecture.
- Cliquez sur **CARTOUCHES** pour replier le mur de cartouches ou le déplier
  de nouveau. Replié à côté des lecteurs, c'est une bande étroite avec une
  flèche pour le déplier et, pendant qu'une cartouche joue, un point rouge.
- Sous les lecteurs, avec la hauteur que demandent ses rangées, les boutons
  rétrécissent quand la fenêtre est basse (jusqu'à une hauteur minimale) pour
  que toutes les rangées configurées tiennent. Une fois le mur de cartouches
  redimensionné, ou à côté des lecteurs, les boutons grandissent pour le
  remplir, et les noms et les temps gardent leur taille au milieu de chaque
  bouton. Dans les deux cas, le mur de cartouches ne défile que lorsque même
  les plus petits boutons ne tiennent pas.

| Aspect du bouton | Signification |
|---|---|
| Point violet | Jingle |
| Point ambre | Effet |
| Point gris | Spot (publicité) |
| Bordure, teinte et point colorés | La couleur de la cartouche, choisie dans les Paramètres : le point prend la couleur au lieu de celle du type |
| ↻ après le type | Joue en boucle |
| ✋ après le type | Arrête les autres cartouches au lancement |
| Fichier avec une croix / un signe d'avertissement | Le fichier est introuvable / ne peut pas être décodé ; survolez la cartouche pour voir la raison et le chemin. Un fichier introuvable est recherché de nouveau toutes les 30 s (`tuning.missing_recheck_ms`). |
| « Vide », grisé | Aucun fichier attribué |

Les cartouches utilisent les mêmes marqueurs que les pistes. Elles démarrent
à leur cue-in et se terminent à leur cue-out, que vous pouvez modifier sur la
forme d'onde d'un lecteur quand le fichier y est chargé.

## Clavier {#keyboard}

Par défaut, **F1**…**F12** lancent les cartouches 1 à 12 de la page affichée,
et **Ctrl+Space** arrête toutes les cartouches (comme **Tout arrêter** ; il
arrête aussi une cartouche que vous pré-écoutez sur le CUE, même si aucune
cartouche ne joue). Voir [Clavier](keyboard.md) pour les modifier.

Le raccourci qui replie ou déplie le mur de cartouches ne fait rien tant que
le mur de cartouches est masqué depuis la barre supérieure (voir
[Disposition](layout.md#show-and-hide-panels)).

## Configurer les cartouches {#setting-up-carts}

Allez dans **Paramètres → Cartouches** :

- **Pages :** créer, renommer, supprimer (la dernière page ne peut pas être
  supprimée) et définir la taille de la grille (lignes × colonnes). Une grille
  plus petite est refusée si elle ferait disparaître des cartouches qui ont
  un fichier. Une taille saisie dans la case à côté d'un curseur s'applique
  quand vous appuyez sur Entrée ou quittez la case ; Échap l'annule.
- **Importer… / Exporter…** enregistrent une page dans un fichier
  `.cartpage.json` et la rechargent, par exemple pour la partager entre
  studios. Les chemins de fichier relatifs sont résolus par rapport au dossier
  du fichier. Les couleurs voyagent avec la page ; un fichier d'une version
  plus ancienne s'importe sans elles.
- **Cartouches :** cliquez sur une cartouche dans la grille, puis définissez
  son nom, son fichier (**Choisir…** ou **Vider**), son type, **Couleur**,
  **En boucle** et **Arrêter les autres cartouches au lancement**.
  **Couleur** propose aucune couleur et huit couleurs (rouge, orange, ambre,
  vert, turquoise, bleu, violet, rose) : donnez une couleur à tous les spots
  et une autre aux indicatifs de la station pour les repérer d'un coup d'œil.
  Une cartouche vide garde sa couleur, et vider le fichier d'une cartouche ne
  l'enlève pas. Une cartouche avec un fichier a aussi **Exclure de la
  normalisation** et **Ajustement**, les réglages de
  [normalisation du loudness](loudness.md#per-track-exclude-and-trim) de sa
  piste, avec la mesure sous le libellé.

Les sorties Main et Cue propres au mur de cartouches se trouvent dans
**Paramètres → Sorties audio** (la ligne **Mur de cartouches**).
