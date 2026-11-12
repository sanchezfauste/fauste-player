# Disposition

Organisez la fenêtre principale pour votre service : donnez aux lecteurs et
au mur de cartouches les tailles que vous voulez, placez le mur de cartouches
sous les lecteurs ou à côté d'eux, masquez les panneaux dont vous ne vous
servez pas et agrandissez-en un un instant. La disposition est enregistrée
avec la session et revient au démarrage suivant ; une nouvelle installation
commence avec tous les lecteurs de la même largeur et le mur de cartouches
sous eux.

## Afficher et masquer des panneaux {#show-and-hide-panels}

Les tuiles à gauche de la barre supérieure, **Grille** et **Musique** (la
[grille](automation.md#the-schedule-column) et le
[remplissage musical](automation.md#the-music-filler), tous deux masqués tant que
vous ne les affichez pas), **P1** … **Pn** et **Cartouches**, montrent quels
panneaux sont à l'écran : une tuile allumée est affichée. Cliquez sur une
tuile pour masquer ou afficher son panneau. Avec de nombreux lecteurs, les
tuiles défilent latéralement (faites tourner la molette de la souris
au-dessus d'elles) et le bouton de disposition reste en vue. Les autres
panneaux prennent la place libérée, et un lecteur affiché de nouveau revient
à sa largeur.

Masquer ne fait que retirer un panneau de l'écran. Un lecteur masqué continue
de jouer, et sa touche numérique, le MIDI, l'API à distance, sa fenêtre CUE
et l'automatisation le pilotent toujours. Une tuile montre l'état de son
panneau, qu'il soit affiché ou masqué : un point rouge tant qu'un lecteur joue
(ou le mur de cartouches tant qu'une cartouche joue) et deux barres ambre,
comme un bouton Pause, tant qu'un lecteur est en pause.

Masquer un lecteur efface sa piste sélectionnée, si bien que **Delete** ne
peut pas retirer une piste que vous ne voyez pas. Les fichiers déposés sur la
fenêtre en dehors d'une liste vont au dernier lecteur que vous avez utilisé,
ou au premier lecteur affiché ; avec tous les lecteurs masqués, ils sont
refusés, et la barre d'état le dit.

La tuile **Bibliothèque**, après elles, ouvre et ferme la [fenêtre de la
bibliothèque](library.md), qui flotte au-dessus des panneaux ; ce n'est pas un
panneau, si bien que la règle de la dernière tuile allumée ci-dessous ne
s'y applique pas. Elle n'est pas affichée sur un téléphone.

Au moins un panneau reste affiché : on ne peut pas cliquer sur la dernière
tuile allumée. Tant que le mur de cartouches est masqué, le raccourci qui le
replie ou le déplie ne fait rien ; cliquez sur **Cartouches** pour l'afficher.

## Redimensionner les lecteurs {#resize-the-players}

Faites glisser l'espace entre deux lecteurs pour transférer de la largeur de
l'un à l'autre. Les autres lecteurs ne bougent pas, et aucun lecteur ne
devient plus étroit que ses commandes ne l'exigent. Double-cliquez sur
l'espace pour redonner la même largeur à tous les lecteurs.

Quand la fenêtre est trop étroite pour que chaque lecteur affiché tienne à sa
taille minimale, la rangée des lecteurs défile latéralement, comme elle l'a
toujours fait.

## Déplacer et redimensionner le mur de cartouches {#move-and-resize-the-cartwall}

Le mur de cartouches se place sous les lecteurs, ou à gauche ou à droite de la
fenêtre sur toute sa hauteur : choisissez **Cartouches en bas**, **Cartouches
à gauche** ou **Cartouches à droite** dans [le menu de
disposition](#the-layout-menu). Sous les lecteurs, il a la hauteur que demandent
ses rangées, au plus six dixièmes de la hauteur entre la barre supérieure et la
barre d'état ; à côté d'eux, il prend d'abord trois dixièmes de la largeur.

Le mur de cartouches n'est jamais plus étroit que son en-tête (**Tout
arrêter**, son titre et un onglet de page) ni que ses colonnes de boutons à
leur taille minimale. Quand la fenêtre est trop étroite pour lui et pour chaque
lecteur affiché à sa taille minimale, la rangée des lecteurs défile
latéralement.

Faites glisser l'espace entre le mur de cartouches et les lecteurs pour
l'agrandir ou le réduire : vers le haut ou le bas quand il est sous les
lecteurs, latéralement quand il est à côté. Les lecteurs gardent la place que
leurs commandes exigent. Double-cliquez sur l'espace pour revenir à la taille
de départ : la hauteur que demandent ses rangées sous les lecteurs, trois
dixièmes de la largeur à côté d'eux. La taille est enregistrée avec la
disposition ; déplacer le mur de cartouches le fait repartir à sa taille de
départ.

Une fois le mur de cartouches redimensionné, ou tant qu'il est à côté des
lecteurs, ses boutons grandissent pour le remplir ; les noms et les temps
gardent leur taille, au milieu de chaque bouton. Un nom passe à la ligne entre
ses mots ; un mot trop long pour son bouton se termine par une ellipse (…) au
lieu d'être coupé sur plusieurs lignes. Quand le mur de cartouches est plus
petit que ce que demandent ses rangées et ses colonnes aux plus petits
boutons, il défile.

Cliquez sur **CARTOUCHES** pour replier le mur de cartouches. Sous les
lecteurs, seul son en-tête reste ; à côté d'eux, il devient une bande étroite
de la hauteur de la fenêtre, avec une flèche pour le déplier de nouveau et,
tant qu'une cartouche joue, un point rouge. Déplié, il retrouve sa taille.
Tant qu'il est replié, l'espace à côté de lui ne fait rien.

## Agrandir un panneau {#maximize-a-panel}

L'en-tête de chaque lecteur, et celui du mur de cartouches, se termine par
deux petits boutons : **Masquer** (un moins) et **Agrandir** (des coins qui
pointent vers l'extérieur ; **Restaurer**, des coins qui pointent vers
l'intérieur, sur le panneau agrandi). Survolez-les pour voir le nom. Ils sont
grisés tant que la disposition est verrouillée, **Masquer** sur le dernier
panneau affiché, et **Agrandir** quand un seul panneau est affiché et sur un
mur de cartouches replié. Les deux mêmes actions se trouvent dans le menu d'un
clic droit sur le fond de l'en-tête (pas sur l'un de ses boutons) :

- **Agrandir** donne toute la place à ce panneau : les autres panneaux
  rétrécissent à leur taille minimale. Ce n'est que pour un instant et ce
  n'est pas enregistré. **Restaurer** dans le même menu y met fin, et aussi
  masquer ou afficher un panneau, verrouiller la disposition, déplacer le mur
  de cartouches, changer le nombre de lecteurs et faire glisser ou
  double-cliquer un espace entre des panneaux ; les tailles d'avant
  reviennent. Un mur de cartouches replié ne peut pas être agrandi, et replier
  un mur de cartouches agrandi y met fin.
- **Masquer** masque le panneau, comme le fait sa tuile dans la barre
  supérieure.

## Le menu de disposition {#the-layout-menu}

Le bouton de disposition, à droite des tuiles, ouvre un menu avec :

- **Cartouches en bas**, **Cartouches à gauche**, **Cartouches à droite** :
  où se place le mur de cartouches ;
- **Verrouiller la disposition** : tant qu'elle est cochée, rien ne modifie la
  disposition par accident : les tuiles, les autres entrées de ce menu, les
  espaces entre les panneaux et les boutons **Agrandir** et **Masquer** des
  en-têtes ne font rien. La lecture, ainsi que le repli et le dépliage du mur
  de cartouches, fonctionnent comme d'habitude. Le verrouillage est enregistré
  avec la disposition ;
- **Lecteurs de même largeur** : tous les lecteurs de la même largeur ;
- **Réinitialiser la disposition** : retour à la disposition d'une nouvelle
  installation.
