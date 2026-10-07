# Paramètres

Ouvrez **Paramètres** dans la barre supérieure. Fermez-les avec **Fermer** ou
`Esc`. La plupart des changements s'appliquent aussitôt et sont enregistrés
automatiquement.

La fenêtre a une seule taille (900 × 640, plus petite sur un petit écran)
quelle que soit la section, et la section défile à l'intérieur. Chaque
section aligne ses libellés sur une seule colonne.

Les sections Lecteurs, Vumètres, Analyse et Raccourcis clavier ont un bouton
**Rétablir les valeurs par défaut** dans leur en-tête. Il demande une
confirmation puis réinitialise uniquement cette section (Lecteurs conserve le
nombre de lecteurs et la langue ; Raccourcis n'a pas d'autre bouton de
réinitialisation). Sorties audio, Playlists, Cartouches, MIDI et À distance
n'en ont pas.

## Redémarrage en attente {#restart-pending}

Certains changements ne prennent effet que lorsque l'application redémarre :
le système audio, la fréquence d'échantillonnage, la taille du tampon (celle
d'un périphérique aussi), les sorties Main et Cue (lecteurs et mur de
cartouches), les périphériques bit-perfect et les réglages DSD. Le nombre de
lecteurs n'en fait pas partie : il s'applique aussitôt.

Une fréquence ou un tampon donné à un périphérique ne compte que s'il change
avec quoi le périphérique s'ouvre : donner à un périphérique la même valeur
que la valeur globale, ou effacer une telle valeur, n'est pas en attente.

Les limites et les réglages du moteur s'appliquent aussi au prochain
démarrage, mais ils se modifient dans le fichier de configuration, application
fermée (voir [Données et sauvegardes](data-and-backups.md)), si bien qu'ils
n'apparaissent jamais en attente.

Tant que l'un d'eux attend, le pied de page des Paramètres indique « Certaines
modifications s'appliquent après un redémarrage. » et propose **Redémarrer
maintenant**, et la barre supérieure affiche une pastille **Redémarrage en
attente**. Survolez la pastille pour voir ce qui attend. Un bref avis (par
exemple, qu'un réglage a été enregistré) peut prendre un instant la place du
texte du pied de page ; **Redémarrer maintenant** reste. Les deux font la même
chose :

- Quand rien n'est à l'antenne, **Redémarrer maintenant** (ou la pastille)
  redémarre aussitôt.
- Quand quelque chose est à l'antenne, la fenêtre qui liste ce qui sonne
  apparaît, avec **Arrêter et redémarrer** ou **Annuler**.

La session est d'abord enregistrée et l'audio et le contrôle MIDI s'arrêtent,
puis l'application redémarre avec le même dossier de données (`FAUSTE_HOME`),
et rien ne passe à l'antenne tout seul ensuite. Si l'application ne peut pas
redémarrer (dans un Flatpak, aussi quand la nouvelle ne démarre pas à temps),
elle le dit ; lancez-la depuis le menu de vos applications.

## Sorties audio {#audio-outputs}

Les changements de cette section attendent un redémarrage : voir
[Redémarrage en attente](#restart-pending).

![Paramètres, Sorties audio, vue Simple : le sélecteur, le système audio, la fréquence d'échantillonnage, la taille du tampon et les sorties Main et Cue de chaque lecteur (ici le système silencieux)](../../images/guide/settings-outputs.png)

En haut, **Afficher** choisit **Simple** ou **Avancé**. Simple montre le
système audio, la fréquence d'échantillonnage, la taille du tampon et les
sorties. Avancé ajoute, pour chaque périphérique qu'utilise une sortie, sa
propre fréquence et son propre tampon, l'interrupteur bit-perfect et le mode
DSD, puis les réglages DSD. Changer de vue ne fait que montrer ou masquer des
lignes : rien n'est modifié ni réinitialisé. Quand Simple masque un réglage
qui est utilisé, une ligne l'indique. Quand plus aucune sortie n'utilise un
périphérique, sa fréquence et son tampon propres, son interrupteur
bit-perfect et son mode DSD sont oubliés au prochain démarrage de
l'application : si une sortie l'utilise de nouveau ensuite, il repart des
valeurs globales. D'ici là, le choisir de nouveau (par exemple après avoir
échangé deux périphériques) les conserve.

| Réglage | Signification |
|---|---|
| Système audio | Le dernier choix, **Aucune sortie (silence)**, ne joue rien : les lignes de temps tournent en temps réel sans carte son (pour une machine qui n'en a pas, ou pour répéter). Linux : PipeWire (dans les versions qui l'incluent), PulseAudio, JACK ou ALSA. Windows : WASAPI, ASIO (dans les versions qui l'incluent) ou JACK. macOS : Core Audio ou JACK. Les systèmes absents de cet ordinateur, ou sans périphérique de sortie (un serveur JACK qui ne tourne pas), sont indiqués comme indisponibles. « Par défaut du système » utilise le premier disponible dans cet ordre. |
| Fréquence d'échantillonnage | La fréquence à laquelle chaque sortie tourne, sauf si un périphérique a la sienne (Avancé) ; les fichiers y sont convertis avec un rééchantillonnage de haute qualité. Les périphériques bit-perfect démarrent à leur fréquence puis suivent les fichiers. |
| Taille du tampon | Trames par bloc audio, sauf si un périphérique a le sien ; la latence qui en résulte est affichée en dessous |
| Sorties par lecteur | Pour chaque lecteur, un périphérique **Main** (antenne) et un périphérique **Cue** (pré-écoute), chacun avec une paire de canaux. Une carte son qui propose plusieurs profils de sortie (ALSA liste frontal, surround, matériel direct…) montre chacun sous la forme *carte — profil* ; deux entrées qui se liraient encore pareil reçoivent l'identifiant de leur périphérique entre parenthèses. Les interfaces multicanaux peuvent porter plusieurs lecteurs sur des paires différentes. |
| Tester Main / Tester Cue | Joue une courte tonalité (1 kHz sur Main, 440 Hz sur Cue, 1,5 s, −18 dBFS) sur la sortie choisie, pour vérifier le câblage avant de passer à l'antenne |
| Mur de cartouches | Les sorties Main et Cue du mur de cartouches. Main prend par défaut la sortie du système. Sans Cue, il n'y a pas de pré-écoute des cartouches. |
| Fréquence d'échantillonnage : *périphérique* (Avancé) | **Global (...)** utilise la fréquence d'échantillonnage ci-dessus ; une valeur donne à ce périphérique sa propre fréquence. Seules les fréquences que le périphérique annonce sont proposées ; une fréquence enregistrée qu'il n'annonce plus reste listée avec une note indiquant qu'il risque de ne pas s'ouvrir (le périphérique revient alors à la fréquence globale). Les valeurs propres ne s'appliquent qu'aux périphériques qu'une sortie nomme, pas à la sortie par défaut du système sauf si une sortie la nomme. |
| Taille du tampon : *périphérique* (Avancé) | **Global (...)** utilise la taille du tampon ci-dessus ; une valeur donne à ce périphérique la sienne, avec sa latence en dessous. Un périphérique qui n'accepte pas sa propre taille de tampon revient à la taille globale, et aussi à la fréquence globale quand il n'accepte pas non plus sa propre fréquence. |
| Bit-perfect : *périphérique* (Avancé) | Un périphérique bit-perfect est ouvert en accès exclusif et suit la fréquence d'échantillonnage de chaque fichier tant que rien n'y joue. L'interrupteur est désactivé là où le périphérique ne peut pas donner l'accès exclusif. Voir [Sortie bit-perfect](bit-perfect.md). |
| DSD : *périphérique* (Avancé) | **Convertir en PCM** (par défaut), **DoP** ou, sous Linux, **DSD natif**. Chaque périphérique le montre ; seuls les modes que le périphérique peut accepter sont proposés, et une ligne en dessous dit pourquoi les autres ne le sont pas. Voir [DSD](bit-perfect.md#dsd). |
| Quand une autre source a besoin d'une sortie DSD (Avancé) | **Poursuivre la piste DSD en PCM** (par défaut), ou **Garder le DSD et couper les autres sources**. Voir [DSD](bit-perfect.md#dsd). |
| Silence DSD (Avancé) | Silence envoyé avant le début d'un flux DSD, après sa fin et lors du passage en PCM, pour que le convertisseur se verrouille sans clic ; 200 ms par défaut, de 0 à 2000. |

Une sortie Cue ne retombe jamais sur la sortie qu'utilise Main, afin que la
pré-écoute n'aille jamais à l'antenne. Un Cue qui désigne un périphérique d'un
système audio que cet ordinateur n'a pas, ou la même sortie (périphérique et
canaux) que Main, signifie « pas de cue ». Quand une sortie Cue est la même
que sa sortie Main, un avertissement en dessous l'indique. Un lecteur sans
sortie Cue, ou dont le Cue est sur sa sortie Main, a son bouton **CUE**
grisé ; le survoler vous invite à choisir une sortie Cue ici. Il en va de
même pour **Pré-écouter sur le CUE** du mur de cartouches.

Si un périphérique disparaît pendant la lecture, les lecteurs conservent leurs
lignes de temps, et le périphérique est rouvert à son retour (voir
[Dépannage](troubleshooting.md)).

## Lecteurs {#players}

![Paramètres, Lecteurs : nombre de lecteurs, mode par défaut, durée du fondu, enchaînement automatique, cue-in et cue-out, alerte de fin de piste et langue](../../images/guide/settings-players.png)

| Réglage | Défaut | Signification |
|---|---|---|
| Langue | Système | Langue de l'interface |
| Nombre de lecteurs | 4 | Colonnes de l'écran principal (un lecteur à l'antenne ne peut pas être retiré) |
| Mode par défaut | CONT | Le mode dans lequel démarrent les lecteurs |
| Durée du fondu | 1000 ms | Utilisée par PLAY pendant l'antenne et par Stop en fondu |
| Enchaînement automatique au point MIX | Activé | Faire chevaucher les pistes en mode continu |
| Utiliser le cue-in et le cue-out | Activé | Désactivé : les lecteurs jouent chaque piste du début à la fin du fichier ; les marqueurs sont conservés et les cartouches utilisent toujours les leurs. Les durées et les totaux de playlist suivent la même plage |
| Alerte de fin de piste | 10 s | Quand le compte à rebours commence à clignoter en rouge |

## Vumètres {#meters}

![Paramètres, Vumètres, avec le vumètre de crête numérique choisi](../../images/guide/settings-meters.png)

Les changements s'appliquent aussitôt. Les Paramètres ne montrent que ce que
le type de vumètre choisi utilise : les vumètres UER, DIN et VU ont l'échelle,
la zone rouge et le comportement que fixe leur norme (seul le niveau
d'alignement se règle), et l'alignement d'un vumètre K-System est son propre
0. Une valeur que vous avez définie est conservée pour quand vous choisirez
de nouveau ce type.

| Réglage | Défaut | Signification |
|---|---|---|
| Type de vumètre | Crête numérique | Comment la barre monte et descend, et son échelle, selon une norme (voir plus bas) |
| Temps de montée, Vitesse de retour | 5 ms, 11,8 dB/s | Uniquement pour **Personnalisé**. Le temps de montée est un temps d'intégration : un train d'ondes de cette durée se lit 2 dB trop bas ; 0 montre chaque crête. |
| True peak | Désactivé | Crête numérique, personnalisé et K-System uniquement. Mesure entre les échantillons, avec le filtre de suréchantillonnage 4× que publie l'ITU-R BS.1770. Il montre les crêtes qui dépassent 0 dBFS après conversion, qu'un vumètre de crête d'échantillon manque. Comme la norme le permet, un clic isolé d'un seul échantillon peut se lire jusqu'à environ 0,3 dB sous sa valeur d'échantillon. |
| Bas de l'échelle | −60 dBFS | Le bas de l'échelle numérique (crête numérique et personnalisé). Les autres vumètres montrent la plage que donne leur norme. |
| Maintien de crête | 2 s | Crête numérique, personnalisé et K-System uniquement : combien de temps le niveau le plus élevé reste allumé ; 0 le désactive. Les vumètres de programme et le VU n'ont pas de maintien. |
| Niveau d'alignement | −18 dBFS | Tous sauf le K-System. Marqué sur l'échelle (UER R68). C'est aussi là que se trouvent le repère TEST UER, le repère −9 DIN et 0 VU. |
| Alerte à partir de | −9 dBFS | Jaune à partir de là (maximum autorisé UER), pour les vumètres de crête numérique et personnalisé |
| Danger à partir de | −3 dBFS | Rouge à partir de là, pour les vumètres de crête numérique et personnalisé. Les autres passent au rouge là où leur échelle le fait : VU à partir de 0 VU, PPM UER et DIN à partir du maximum autorisé (UER +9, DIN 0), le K-System à partir de +4. |
| Affichage du loudness | Court terme | Le loudness sous le vumètre : désactivé, momentané (les 400 dernières ms) ou court terme (les 3 dernières s), EBU R128 |
| Loudness cible | −23 LUFS | L'affichage est vert à ±1 LU (EBU R128) |

| Type de vumètre | Norme | Comportement |
|---|---|---|
| Crête numérique | IEC 60268-18 | Montre chaque crête aussitôt ; retombe de 20 dB en 1,7 s |
| PPM UER | IEC 60268-10 type IIb | Les crêtes plus courtes qu'environ 10 ms se lisent plus bas (un train d'ondes de 10 ms se lit environ 1,6 dB trop bas, un de 0,5 ms environ 18 dB trop bas), dans les tolérances de l'EBU Tech 3205 ; retombe de 24 dB en 2,8 s |
| PPM DIN | IEC 60268-10 type I | Pareil avec un temps d'intégration de 5 ms ; retombe de 20 dB en 1,5 s |
| VU | IEC 60268-17 | Le niveau moyen, avec le mouvement d'aiguille d'un vumètre VU : 99 % en 300 ms, avec un léger dépassement ; une sinusoïde lit son niveau de crête |
| K-20, K-14, K-12 | K-System | Deux sections : la moyenne (RMS, 600 ms) en barre pleine et la crête (retombe de 26 dB en 3 s) atténuée au-dessus. 0 est 20, 14 ou 12 dB sous la pleine échelle ; vert sous 0, ambre de 0 à +4, rouge au-dessus. Le K-12 convient à la diffusion, le K-14 et le K-20 à un programme plus dynamique. |
| Personnalisé | — | Votre temps de montée et votre vitesse de retour |

Chaque vumètre utilise l'échelle de sa norme, avec ses repères entre les
canaux :

| Vumètre | Échelle |
|---|---|
| Crête numérique, personnalisé | −60 … 0 dBFS, repères tous les 10 dB jusqu'à −40 et tous les 5 dB au-dessus ; les 20 dB du haut occupent la moitié de la hauteur |
| PPM UER | −12 … +12 autour du niveau d'alignement (TEST), tous les 4 dB ; les niveaux plus faibles restent en bas |
| PPM DIN | −50 … +5, où 0 est 9 dB au-dessus du niveau d'alignement (−9 dBFS par défaut) |
| VU | −20 … +3 VU, 0 VU au niveau d'alignement ; la barre se déplace proportionnellement à la tension, comme l'aiguille |
| K-System | de +20, +14 ou +12 (0 dBFS) jusqu'à −60 ; régulier en dB jusqu'à −24 |

## Analyse {#analysis}

![Paramètres, Analyse : les seuils des marqueurs automatiques](../../images/guide/settings-analysis.png)

Les seuils décrits dans [Marqueurs et mixage](markers-and-mixing.md).
**Réanalyser toutes les pistes** relance l'analyse pour toute la
bibliothèque ; les marqueurs manuels sont conservés.

Après une mise à jour dont l'analyse a changé, les pistes analysées par la
version antérieure gardent leurs marqueurs et leurs formes d'onde, qui
fonctionnent toujours. Au démarrage, Fauste Player indique combien il y en a
et propose **Analyser maintenant** ou **Plus tard** ; **Analyser les pistes
obsolètes (N)** ici fait la même chose à tout moment. Les pistes sur les
lecteurs sont mises à jour de toute façon, au moment où elles sont affichées,
de même que les pistes des cartouches qui n'ont pas de format enregistré (une
cartouche ne joue en bit-perfect que si son format est connu). Les pistes dont
le fichier est introuvable ne sont pas comptées tant que le fichier n'est pas
revenu.

## Playlists {#playlists}

![Paramètres, Playlists : le dossier de musique, les playlists et les colonnes du tableau](../../images/guide/settings-playlists.png)

- **Dossier de musique :** là où démarrent les boîtes de dialogue de fichier.
- **Nouvelle playlist**, **renommer** (modifiez le nom et appuyez sur Enter ;
  Esc annule) et **supprimer** (icône de corbeille).
- **Importer M3U / PLS…** crée une nouvelle playlist à partir d'un fichier de
  playlist. **M3U** sur chaque ligne l'exporte en M3U8. Voir
  [Playlists](playlists.md).
- **Colonnes du tableau :** quelles colonnes les tableaux de pistes montrent
  et dans quel ordre, pour chaque lecteur : une case à cocher par colonne
  (Titre et Durée ne peuvent pas être désactivées), des flèches haut et bas
  pour celles qui sont affichées, et **Colonnes par défaut**. Voir
  [Playlists](playlists.md).

**Langue :** une liste déroulante : **Système** (suivre le système
d'exploitation), puis chaque langue dans laquelle l'interface est disponible,
chacune sous son propre nom (l'anglais d'abord, puis par ordre alphabétique :
par exemple Español). L'interface change aussitôt. Une langue du système sans
traduction propre utilise la plus proche (le français canadien utilise le
français, le portugais brésilien utilise le portugais), et l'anglais sinon.
Une langue présente dans le fichier de paramètres que l'interface n'a pas
s'affiche comme **Système** et suit le système d'exploitation.

L'anglais et l'espagnol sont écrits à la main. Les autres traductions ont été
générées par IA et peuvent contenir des erreurs ; quand l'une d'elles est
utilisée, **À propos** l'indique. Les corrections de locuteurs natifs sont les
bienvenues sous forme d'issues ou de pull requests.

## Cartouches {#cartwall}

![Paramètres, Cartouches : les pages, la grille et l'éditeur de la cartouche sélectionnée](../../images/guide/settings-cartwall.png)

Pages, taille de la grille, éditeur de cartouche, et import et export de pages
de cartouches. Voir [Mur de cartouches](cartwall.md).

## Raccourcis clavier {#keyboard-shortcuts}

![Paramètres, Raccourcis clavier : chaque action de lecteur avec sa touche, et Retirer à côté de celles qui sont associées](../../images/guide/settings-shortcuts.png)

Voir [Clavier](keyboard.md).

## MIDI {#midi}

![Paramètres, MIDI, avec le contrôle MIDI désactivé](../../images/guide/settings-midi.png)

Activez les surfaces de contrôle MIDI, consultez les ports d'entrée et
apprenez une commande pour chaque action de lecteur. Voir
[Surfaces de contrôle MIDI](midi.md).

## À distance {#remote}

![Paramètres, À distance, avec l'API HTTP à l'écoute sur cet ordinateur](../../images/guide/settings-remote.png)

Contrôle à distance par le réseau, pour les pages web, les applications pour
téléphone, l'automatisation et les surfaces de contrôle. Voir
[Contrôle à distance](remote-control.md).

- **Autoriser le contrôle à distance par HTTP**, son **adresse** et son
  **port**, et une ligne qui indique s'il est à l'écoute.
- **Jeton**, obligatoire au-delà de cet ordinateur. **Générer** en crée un
  aléatoire, **Afficher** le révèle et **Copier** le place dans le
  presse-papiers. Un avertissement apparaît quand l'adresse porte au-delà de
  cet ordinateur et qu'il n'y a pas de jeton.
- **Pages web autorisées à utiliser l'API** : une origine par ligne.
- **Autoriser le contrôle par OSC**, son **adresse** et son **port**, et les
  **expéditeurs autorisés** (adresses ou sous-réseaux, un par ligne).
- **Publier les temps toutes les** : à quelle fréquence les temps écoulés et
  restants sont envoyés pendant que quelque chose joue.

Les champs de texte et les nombres s'appliquent quand vous les quittez, ce qui
inclut l'ouverture d'une autre section ou la fermeture des Paramètres ; Esc
annule ce que vous étiez en train de saisir. Une valeur invalide est
corrigée, et le champ montre ce qui a été conservé.
