# Automatisation

L'automatisation garde la station à l'antenne quand personne n'est au studio.
Cette version a le **remplissage musical**, un lecteur qui choisit la musique
tout seul, à partir des règles que vous fixez, et qui ne s'arrête jamais faute
de liste, et la colonne **Grille**, qui exécute les blocs horaires du jour,
avec l'éditeur où vous les faites.
Les lecteurs que vous utilisez déjà (P1 … Pn) ne sont pas touchés : ils
restent les vôtres pour les émissions que vous préparez et jouez en direct.

## La colonne Grille {#the-schedule-column}

La colonne **Grille** joue les **blocs** du jour : des groupes d'éléments avec
une heure de début (un indicatif horaire, des spots, une émission). Affichez-la
avec la tuile **Grille** de la barre supérieure, comme vous affichez un
lecteur.

![La colonne Grille en marche, à côté de la colonne Musique : les blocs du jour par heure de début, chacun avec son indicatif de la station (l'horloge le marque comme ponctuel), des spots et la rotation que le remplissage musical joue ensuite ; la chanson du remplissage à l'antenne et les suivantes](../../images/guide/automation-columns.png)

Sa liste est la grille, groupée par blocs : l'heure de début de chaque bloc,
puis ses éléments. Elle montre aujourd'hui, le dernier bloc d'hier et le
premier de demain. Un bloc modifié pour un jour le dit. L'élément à l'antenne
et le suivant sont marqués comme sur un lecteur ; les éléments déjà diffusés
sont atténués, et un élément qui n'a pas été diffusé (**Non diffusé**) est
barré. Une horloge marque un élément **ponctuel**, qui ne joue qu'à son heure.

Au-dessus de la liste :

- **Démarrer** lance la grille : chaque bloc passe à son heure, et le
  remplissage musical joue entre les blocs. Rien ne démarre tout seul : après
  chaque démarrage de l'application, l'automatisation est arrêtée jusqu'à ce
  que vous appuyiez sur Démarrer.
- **Arrêter après l’actuel** laisse se terminer ce qui est à l'antenne, puis
  plus rien ne joue tout seul.
- **En marche** ou **Arrêtée** dit dans quel état elle est.

Pour changer ce qui joue ensuite, double-cliquez sur un élément, ou faites un
clic droit dessus et choisissez **Passer ensuite** ; il suit la chanson à
l'antenne. **Revenir à l’horloge**, dans le même menu, laisse la grille
choisir de nouveau d'après l'heure.

## Modifier la grille {#editing-the-schedule}

**Modifier la grille**, sur la colonne Grille, ouvre l'éditeur. Il a trois
onglets, et **Rotations…** vous mène à Paramètres → Automatisation, où l'on
fait les rotations.

![L'éditeur de grille, onglet Modèles : les modèles Base et Weekend à gauche ; les blocs de Base à droite, chacun avec son début, ses éléments, Ajouter depuis une catégorie et la rotation de la musique qui suit](../../images/guide/schedule-templates.png)

- **Modèles.** Un modèle est un jour de blocs ; une nouvelle installation en
  a un, « Base », pour tous les jours. Créez, dupliquez, renommez ou
  supprimez des modèles à gauche ; les blocs de celui qui est choisi sont à
  droite. Saisissez le début d'un bloc sous la forme `HH:MM:SS` (une heure qui
  n'existe pas reste dans le champ avec une note, et rien ne change). Glissez
  des pistes de la bibliothèque sur un bloc pour les y ajouter ; **Ajouter
  depuis une catégorie** ajoute un élément qui joue une piste de cette
  catégorie, une différente à chaque fois ; **Musique ensuite** choisit la
  rotation que le remplissage musical joue après le bloc (la rotation par
  défaut si aucune n'est choisie). Chaque élément peut être marqué
  **Ponctuel**, déplacé ou retiré. Quand les éléments d'un bloc durent plus
  que le temps jusqu'au bloc suivant, le bloc dit de combien, en ambre ; vous
  pouvez quand même le garder ainsi.
- **Semaine type.** Quel modèle chaque jour de la semaine utilise.

  ![L'onglet Semaine type : Base du lundi au vendredi, Weekend le samedi et le dimanche](../../images/guide/schedule-week.png)

- **Jours.** Choisissez une date dans le calendrier (les dates avec des
  changements sont en ambre). Une date peut utiliser un autre modèle que celui
  de son jour de la semaine (un jour férié un jeudi peut utiliser celui du
  weekend). Ses blocs peuvent être modifiés pour cette date seulement : le
  changement est marqué, et **Revenir au modèle** l'annule. **Pas ce
  jour-là** retire un bloc de cette date seule, et **Ajouter un bloc ce
  jour-là** en ajoute un. Les modifications ultérieures du modèle atteignent
  encore les blocs que vous n'avez pas changés à cette date.

  ![L'onglet Jours : le calendrier avec aujourd'hui choisi et les dates avec des changements en ambre ; le premier bloc d'aujourd'hui a un spot de plus, marqué « modifié pour ce jour », avec Revenir au modèle](../../images/guide/schedule-days.png)

## Le remplissage musical {#the-music-filler}

Le remplissage musical est la colonne **Musique**. Elle est masquée tant que
vous ne l'affichez pas : cliquez sur la tuile **Musique** à gauche de la barre
supérieure. Elle se place à gauche de P1.

Elle ressemble à un lecteur et fonctionne comme lui : la grande horloge, la
forme d'onde, les boutons de transport, le fader et la liste de pistes. La
différence est la liste :

- elle **se remplit toute seule** à partir d'une **rotation** (plus bas),
  toujours environ une demi-heure à l'avance (*avance*, dans Paramètres →
  Automatisation) ;
- elle **se rogne toute seule** : elle garde les 50 dernières pistes jouées et
  retire les plus anciennes, si bien qu'elle peut tourner pendant des
  semaines ;
- avant un bloc de la grille, elle peut **échanger des chansons qu'elle a
  choisies elle-même** contre d'autres dont la durée convient au temps restant
  (voir plus bas) ; les chansons que vous avez ajoutées ne sont jamais
  touchées ;
- vous pouvez toujours la **modifier à la main** : glisser des pistes de la
  bibliothèque, réordonner les lignes, retirer une chanson dont vous ne voulez
  pas. Une ligne que vous ajoutez est marquée d'une main et n'est jamais
  remplacée ; une ligne que vous retirez ne compte pas comme jouée.

Ses sorties se règlent dans Paramètres → Sorties audio, sur la ligne
**Musique**, comme pour tout lecteur. Elle n'a pas de numéro, ni de raccourcis
clavier ou d'assignations MIDI, et le contrôle à distance ne la voit pas.

Appuyez sur **PLAY** pour la démarrer et sur **Arrêter après l’actuel** pour
laisser se terminer la chanson à l'antenne puis s'arrêter. Comme tout lecteur,
elle ne démarre jamais toute seule, pas même après un redémarrage.

Au-dessus de sa liste, là où un lecteur a ses onglets de playlist, la colonne
Musique montre la **rotation** qu'elle joue. Choisissez-en une autre là : les
chansons qu'elle avait en file sont remplacées, celles que vous avez ajoutées
restent. Quand la rotation ne trouve rien à jouer (ses catégories sont vides,
ou tous les fichiers sont manquants), la barre le dit.

## Rotations {#rotations}

Une **rotation** est une courte liste d'**étapes** que le remplissage joue à
tour de rôle, puis de nouveau depuis la première. Chaque étape nomme d'où
viennent ses pistes (une catégorie de la bibliothèque avec ses
sous-catégories, ou toutes les pistes d'un type) et combien elle en joue à
chaque fois, sous la forme d'une plage :

| Étape | Joue |
|---|---|
| Succès actuels | 2–3 |
| Indicatifs de la station | 1 |
| Or | 0–1 |

Avec une plage, un indicatif de la station vient après deux chansons une fois
et après trois la fois suivante. Une plage qui part de **0** permet à une
étape de sauter certains tours : avec Or à 0–1, une chanson d'or joue après
l'indicatif de la station tantôt oui, tantôt non. Le maximum est toujours d'au
moins 1. L'une d'elles est celle **par défaut** : une nouvelle
installation en a une, **Music**, qui joue toute votre musique.

Les rotations se font dans Paramètres → Automatisation :

![Paramètres, Automatisation : les rotations Music (par défaut) et Hits, Music sélectionnée avec son unique étape sur toute la musique, ses nombres Au moins et Au plus, et les règles du remplissage musical](../../images/guide/settings-automation.png)

## Comment il choisit {#how-it-chooses}

Dans une étape, le remplissage préfère les pistes qui sont restées le plus
longtemps sans passer à l'antenne, avec un peu de hasard parmi elles pour que
l'ordre ne devienne jamais un cycle fixe. Il tient aussi à distance :

- **la même piste :** elle peut rejouer une fois 3 heures écoulées, ou quand la
  moitié des autres pistes de son étape sont passées depuis, selon ce qui
  arrive en premier ;
- **le même artiste :** pas avant 30 minutes.

Tout ce qui est passé à l'antenne compte, qu'il vienne du remplissage, d'un
lecteur ou du mur de cartouches ; la pré-écoute ne compte pas. Quand une petite
catégorie ne peut pas tenir les règles, le remplissage cède plutôt que de se
taire : d'abord sur l'artiste, puis sur la piste. Paramètres → Automatisation a
toutes les valeurs, et c'est là que l'on fait, renomme et modifie les
rotations (voir [Paramètres](settings.md#automation)).

Avant un bloc de la grille, le remplissage choisit aussi ses dernières
chansons d'après leur durée, pour que la musique se termine proprement : la
chanson qui s'estompe au début du bloc perd au plus 30 secondes, et aucune
chanson ne démarre si elle ne devait jouer que moins de 45 secondes. Quand
aucune chanson de la rotation ne satisfait les deux, c'est celle qui perd le
moins qui joue. Ces deux valeurs ne peuvent être changées pour l'instant que
dans le fichier de configuration (`automation.max_fade_cut_secs` et
`automation.min_song_play_secs`).

Un **Indicatif horaire** est un type de piste à part entière (Bibliothèque →
Type ▸ Indicatif horaire), pour l'identification de la station en début
d'heure.
