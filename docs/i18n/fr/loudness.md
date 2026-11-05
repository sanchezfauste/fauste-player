# Normalisation du loudness

Les pistes viennent de nombreuses sources et sont masterisées à des niveaux
très différents : un single pop récent peut sonner deux fois plus fort qu'un
enregistrement des années 1970 ou qu'une promo parlée. La **normalisation du
loudness** joue chaque piste et chaque cartouche au même loudness perçu, si
bien que l'opérateur n'a pas à manœuvrer le fader d'une piste à l'autre.

Elle est **activée pour les nouvelles installations** ; une configuration
existante garde son réglage actuel (toute configuration enregistrée le stocke
déjà). Modifiez-la dans [Paramètres → Loudness](settings.md#loudness).

![Paramètres, Loudness : normalisation activée, avec la cible, le plafond de true peak, le gain maximal et l'ajustement maximal à leurs valeurs par défaut](../../images/guide/settings-loudness.png)

## Comment elle fonctionne {#how-it-works}

**Un gain fixe par piste.** Chaque piste reçoit un seul gain, en dB, décidé au
chargement de la piste et conservé jusqu'à son arrêt. Rien n'est compressé ni
limité : toute la piste est rendue plus forte ou plus faible de la même
quantité, si bien que sa propre dynamique reste telle qu'elle a été
masterisée. Le fader du lecteur agit par-dessus, comme avant.

**La mesure.** L'analyse en arrière-plan mesure chaque fichier une fois :

- son **loudness intégré**, en LUFS : à quel point le fichier entier sonne
  fort, comme le définissent l'ITU-R BS.1770 et l'EBU R128 (les passages
  faibles et le silence sont écartés par le gating, si bien qu'un long fondu
  de sortie ou une pause ne l'abaisse pas) ;
- son **true peak**, en dBTP : le niveau le plus élevé que le signal atteint
  entre les échantillons une fois converti en analogique.

Les nouveaux fichiers sont mesurés lors de leur première analyse. Les fichiers
ajoutés avant que le loudness soit mesuré le sont par un **passage en
arrière-plan** qui tourne pendant que l'analyse normale n'a rien à faire, sans
toucher à leurs marqueurs. Il tourne que la normalisation soit activée ou non,
si bien que les mesures sont prêtes quand vous l'activez. **Réanalyser** (dans
le menu de la playlist) mesure de nouveau un fichier, en le relisant même si sa
taille et sa date n'ont pas changé. Un fichier plus court que 0,4 s n'est
jamais mesuré, et un fichier silencieux n'a pas de loudness : les deux sont
lus tels quels.

**Le gain.** Le gain amène le loudness mesuré à la **Cible**, plus
l'**Ajustement** propre à la piste (voir [Par piste](#per-track-exclude-and-trim)) :

> gain = cible − loudness mesuré + ajustement

Une piste plus forte que la cible est baissée autant qu'il le faut. Une piste
plus faible est relevée, mais jamais de plus que le **Gain maximal**, et jamais
au point que son true peak dépasse le **Plafond de true peak** : il n'y a pas
de limiteur, c'est donc le plafond qui empêche une piste relevée d'écrêter.
Une piste dont le true peak est déjà au-dessus du plafond n'est pas relevée du
tout.

Exemples, avec les valeurs par défaut (cible −16 LUFS, plafond −1 dBTP, gain
maximal 12 dB) :

| Piste | Mesuré | True peak | Gain |
|---|---|---|---|
| Un master moderne fort | −9,0 LUFS | +0,5 dBTP | −7,0 dB |
| Un master plus ancien et plus faible | −20,0 LUFS | −10,0 dBTP | +4,0 dB |
| Une piste faible avec des crêtes élevées | −22,0 LUFS | −3,0 dBTP | +2,0 dB (limité par le plafond) |
| Un enregistrement très faible | −35,0 LUFS | −20,0 dBTP | +12,0 dB (limité par le gain maximal) |

**Quand le gain est décidé.** Au chargement d'une piste : préparée comme
suivante d'un lecteur, démarrée, chargée en pause, pré-écoutée sur le CUE, ou
lancée comme cartouche. Une piste en lecture ou en pause **garde son gain**
jusqu'à son arrêt, malgré les déplacements et les pauses. Modifier les
réglages, la mesure d'une piste ou son ajustement s'applique la prochaine fois
que la piste est chargée ; la piste suivante qui attend déjà sur un lecteur est
préparée de nouveau avec le nouveau gain, si bien qu'elle démarre avec lui.

**Pré-écoute.** La pré-écoute CUE et celle des cartouches jouent une piste avec
le même gain que celui qu'elle reçoit à l'antenne, si bien que ce que vous
entendez au casque correspond au programme.

**Vumètres.** Les vumètres et leur affichage du loudness mesurent après le
gain, ils montrent donc le niveau normalisé. Le **Loudness cible** propre aux
vumètres (Paramètres → Vumètres) ne fait que colorer l'affichage ; il ne change
aucun gain.

## Sorties bit-perfect {#bit-perfect-outputs}

Un lecteur ou une cartouche dont la sortie est [bit-perfect](bit-perfect.md)
n'est **jamais normalisé** : il joue toujours au gain unitaire (0 dB), quels
que soient les réglages, parce que tout gain changerait les échantillons que
reçoit le périphérique. Avec la normalisation activée, son en-tête affiche un
badge **0.0 dB** ambre dont l'infobulle dit « Sortie bit-perfect : les pistes
sont lues sans modification. » Il en va de même pour la pré-écoute sur un
périphérique Cue bit-perfect, alors que la sortie Main du lecteur, si elle
n'est pas bit-perfect, reçoit quand même le gain.

Une piste qui est déjà en lecture (ou en pause) quand son périphérique devient
bit-perfect garde le gain avec lequel elle a démarré, et le badge **BP** reste
éteint pour elle. La piste suivante est lue à 0 dB et en bit-perfect. Une piste
DSD dans cette situation est lue convertie en PCM avec son gain, jamais comme
un flux DSD intact.

## Le gain dans l'en-tête du lecteur {#the-gain-in-the-player-header}

Tant qu'un lecteur est en lecture ou en pause, un badge à gauche de **BP**
montre le gain de la piste à l'antenne. Le badge a une bordure de sa couleur et
toujours la même largeur. Survolez-le pour la raison.

| Badge | Signification |
|---|---|
| **+4.0 dB**, **−7.0 dB** (bleu) | La piste est normalisée avec ce gain. L'infobulle est le détail du gain décrit plus bas. Quand les réglages, la mesure ou l'ajustement ont changé après le chargement de la piste, elle dit seulement que la piste garde le gain fixé à son chargement, et montre ce gain. |
| **0.0 dB** (ambre) | La piste est lue telle quelle. L'infobulle est le même détail, et sa ligne **Gain automatique** dit pourquoi : elle n'a pas encore de mesure (ou est trop courte pour être mesurée), la sortie est bit-perfect, ou la piste est exclue de la normalisation. |
| Pas de badge | La normalisation était désactivée au démarrage de la piste, la piste est silencieuse, ou le lecteur est arrêté. Un lecteur bit-perfect n'en affiche pas non plus tant que la normalisation est désactivée. |

Le badge montre le gain que le lecteur a demandé au démarrage de la piste.

### Le détail du gain {#the-gain-breakdown}

Partout où le programme explique le gain d'une piste, il montre le même bloc de
lignes, libellé à gauche et valeur à droite : l'infobulle du badge de gain, la
fenêtre de loudness (sous ses commandes), l'éditeur de cartouche et l'infobulle
de piste du tableau de la playlist.

| Ligne | Signification |
|---|---|
| **Loudness intégré** | Le loudness mesuré de la piste, en LUFS (ou « non mesuré »). |
| **True peak** | Son true peak le plus élevé, en dBTP (seulement quand il est mesuré). |
| **Cible** | Le loudness auquel chaque piste est amenée (Paramètres → Loudness). |
| **Gain automatique** | Cible moins loudness mesuré, avant l'ajustement et toute limite, ou « aucun » et pourquoi : exclue, sortie bit-perfect, normalisation désactivée, pas encore mesurée, ou silencieuse. |
| **Ajustement** | L'ajustement manuel de la piste en vigueur, quand il n'est pas 0 (avec la valeur enregistrée et le maximum quand un **Ajustement maximal** abaissé le limite). |
| **Gain voulu** et **Limité par** | Seulement quand le plafond de true peak ou le gain maximal a retenu le gain : le gain automatique plus l'ajustement (affiché quand il y a un ajustement) et la limite qui l'a arrêté. |
| **Gain résultant** | Toujours en dernier, sous un filet et en gras : le gain avec lequel la piste est lue. 0.0 dB quand elle n'est pas normalisée. |

Quand la normalisation est désactivée, une note au-dessus du gain résultant dit
que rien ne change à l'antenne et que l'ajustement s'applique dès qu'elle est
activée.

## Par piste : exclure et ajuster {#per-track-exclude-and-trim}

Certaines pistes ne doivent pas suivre la cible : un indicatif de station
masterisé fort exprès, une tonalité de test, une piste dont la mesure ne
correspond pas à ce qu'on entend (la parole sonne souvent plus fort que la
musique au même LUFS). Chaque piste a deux réglages pour cela :

- **Exclure de la normalisation :** la piste est toujours lue telle quelle.
- **Ajustement :** des dB ajoutés au gain de la piste, de − à + l'**Ajustement
  maximal** (Paramètres → Loudness), par pas de 0,5 dB. Double-cliquez sur le
  curseur pour le remettre à 0. L'ajustement est grisé tant que la piste est
  exclue.

Le curseur est un **ajustement** : un décalage par-dessus le gain automatique,
il affiche donc 0 dB pour toute piste que vous n'avez pas modifiée. La fenêtre
et l'éditeur de cartouche montrent donc le [détail du gain](#the-gain-breakdown)
avec leurs commandes : la ligne **Gain résultant** suit le curseur pendant que
vous le déplacez. Dans la fenêtre, les commandes viennent d'abord et le détail
dessous, si bien que le curseur reste où il est pendant que le détail grandit
ou rétrécit. Tant que la piste sonne, la fenêtre ajoute qu'une modification
s'applique à partir de son prochain chargement.

Où les modifier :

- **Sur une forme d'onde :** faites un clic droit sur la forme d'onde d'un
  lecteur (ou celle de sa fenêtre CUE) et choisissez **Loudness…**. La fenêtre
  nomme la piste et montre sa mesure. Elle continue de modifier cette piste
  même quand le lecteur passe à une autre ; fermez-la avec son bouton **×**.
  Elle se ferme d'elle-même si la piste est retirée.
- **Sur une ligne de playlist :** faites un clic droit sur la ligne et
  choisissez **Loudness…** : la même fenêtre, pour la piste de cette ligne.
- **Dans l'éditeur de cartouche :** Paramètres → Cartouches, cliquez sur une
  cartouche : les deux mêmes commandes se trouvent sous **Arrêter les autres
  cartouches au lancement**, avec la mesure sous le libellé.
- **À distance :** `PUT /api/v1/tracks/<id>/loudness` (voir
  [Contrôle à distance](remote-control.md)).

Les réglages appartiennent à la piste et sont enregistrés avec elle. Une
modification s'applique la prochaine fois que la piste est chargée, comme
toute autre modification.

**L'infobulle de piste** du tableau de la playlist montre les tags de la piste
et, sous un filet, le même [détail du gain](#the-gain-breakdown), calculé pour
le lecteur auquel appartient cette liste (un lecteur bit-perfect la joue sans
modification).

## Choisir les réglages {#choosing-the-settings}

- **Cible.** −16 LUFS convient à la plupart des programmes musicaux à
  l'antenne et en ligne. −23 LUFS (EBU R128) ou −24 LUFS (ATSC A/85) laissent
  plus de marge, mais les pistes faibles demandent alors de plus grands gains,
  que le plafond et le gain maximal peuvent retenir. Choisissez une cible
  proche du niveau auquel se trouve déjà votre matériel.
- **Plafond de true peak.** −1 dBTP laisse de la place pour la conversion et
  pour les encodeurs avec perte en aval. Abaissez-le quand un encodeur de flux
  écrête.
- **Gain maximal.** Empêche un enregistrement très faible (ou presque
  silencieux) d'être remonté jusqu'au bruit. 0 dB ne fait que baisser les
  pistes fortes.
- **Ajustement maximal.** Jusqu'où l'ajustement par piste peut aller. L'abaisser
  limite les ajustements déjà réglés sans les changer, jusqu'à ce que vous
  modifiiez de nouveau le loudness de cette piste : la modification enregistre
  alors l'ajustement limité.

## Les normes derrière les valeurs par défaut {#standards-behind-the-defaults}

- **Mesure.** Loudness intégré et true peak tels que définis par l'ITU-R
  BS.1770 (pondération K, avec gating).
- **Cible −16 LUFS.** Elle suit les recommandations de l'AES pour le streaming
  et la diffusion en ligne. L'AES TD1008 (2021) remplace l'AES TD1004.1.15-10
  précédente, qui recommandait −16 à −20 LUFS pour le streaming. Ce n'est pas
  une valeur de radiodiffusion : l'EBU R128 utilise −23 LUFS et l'ATSC A/85
  utilise −24 LKFS. Utilisez l'une d'elles quand une spécification de livraison
  l'exige (EBU R128 pour l'échange de programmes de radiodiffusion européens,
  ATSC A/85 pour la télévision numérique américaine), et −16 pour un flux ou un
  programme en ligne.
- **Plafond de true peak −1 dBTP.** Le true peak maximal de l'EBU R128, aussi
  utilisé par les recommandations de l'AES. Les encodeurs avec perte peuvent
  dépasser au décodage, si bien qu'un flux envoyé par un encodeur à faible
  débit peut demander un plafond plus bas, tel que −2 dBTP (une pratique
  courante, qui ne fait pas partie de ces normes).
- **Gain maximal et ajustement maximal, 12 dB.** Des choix de produit sans
  norme derrière : assez pour remonter un enregistrement faible et corriger
  une piste à l'oreille, pas assez pour relever le bruit ou pour que
  l'ajustement l'emporte sur la cible.
- **La normalisation est activée pour les nouvelles installations.** Mettre à
  jour l'application conserve le réglage que votre configuration a déjà, si
  bien qu'une installation qui l'avait désactivée le reste jusqu'à ce que vous
  l'activiez.
- **Le loudness cible des vumètres** (Paramètres → Vumètres) est un réglage
  distinct, par défaut la valeur R128 de −23 LUFS. La normalisation ne
  l'utilise pas.
