# Android

Fauste Player fonctionne aussi sur les téléphones et tablettes Android
(Android 10 ou ultérieur), sur le même cœur de diffusion que l'ordinateur : les
lecteurs, les playlists, le mur de cartouches, les marqueurs, la normalisation
du loudness et chaque règle de lecture fonctionnent de la même façon.

## Installation {#install}

Une version qui contient l'application a
`fauste-player-<version>-android.apk` sur sa page **Releases** (avec son
`.sha256`). Ouvrez-le sur le téléphone et autorisez votre navigateur ou votre
gestionnaire de fichiers à installer des applications quand Android le
demande. Pour mettre à jour, installez l'APK plus récent de la même façon : vos
playlists et vos réglages restent. L'application n'est pas encore sur Google
Play.

La version 0.13.0 et les précédentes s'installent comme une autre application,
donc un APK plus récent ne les met pas à jour : il apparaît comme un second
Fauste Player. Désinstallez l'ancienne une fois la nouvelle configurée. Ses
playlists et ses réglages ne sont pas repris.

## Premier démarrage {#first-start}

- **Accès à la musique.** Autorisez l'accès à la musique (et aux notifications,
  sous Android 13 et ultérieur). Sans cela, le navigateur de musique reste
  vide.
- **La diffusion continue.** Tant qu'un lecteur ou une cartouche joue, une
  notification reste affichée et l'audio continue écran éteint ou application
  en arrière-plan. Comme sur l'ordinateur, rien ne passe à l'antenne tout seul
  après un démarrage ou un plantage.
- **Retour et fermeture.** Retour ferme d'abord ce qui est ouvert (le
  navigateur de musique, les Paramètres, À propos, un lecteur ouvert, une page
  de l'onglet Bibliothèque). Quand il n'y a plus rien à fermer, Retour met
  l'application de côté tant que quelque chose sonne (un lecteur, une
  cartouche ou une pré-écoute CUE) et la lecture continue ; quand rien ne
  sonne, il enregistre et ferme l'application. Balayer l'application dans le
  sélecteur d'applications fait la même chose : un balayage n'arrête jamais ce
  qui joue, et l'application se ferme dès qu'elle devient silencieuse.
- **Le son des autres applications.** Quand une autre application prend
  l'audio, Fauste Player continue de jouer et le dit dans la barre d'état en
  bas.

## L'écran {#the-screen}

Sur un **téléphone**, la barre supérieure a quatre onglets, **Direct**,
**Cartouches**, **Playlists** et **Bibliothèque**, affichés sous forme d'icônes
(celui qui est choisi aussi avec son nom quand il y a de la place), puis le
**cadenas** et **⋮** (Plus : Paramètres et À propos). L'horloge s'affiche
quand il y a de la place pour elle. Tourné sur le côté, l'écran Playlists place
ses pastilles de lecteur (P1, P2…) dans la barre supérieure, et la barre
d'état en bas ne s'affiche que lorsqu'elle a quelque chose à dire.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Direct sur un téléphone : deux lecteurs sous forme de cartes, l'une entière et le début de la suivante, et le mur de cartouches en dessous">
<img src="../../images/android/2-carts.png" width="180" alt="Cartouches sur un téléphone : la page General tournée à la verticale, deux cartouches par rangée">
<img src="../../images/android/3-playlists.png" width="180" alt="Playlists sur un téléphone : les pastilles P1 à P4, les onglets de playlist, la liste des pistes et la carte du lecteur en bas">
<img src="../../images/android/4-music.png" width="180" alt="La musique de l'appareil sur un téléphone : le dossier Demo avec trois morceaux cochés et la barre pour les ajouter à la playlist Morning de P1">
<img src="../../images/android/5-library.png" width="180" alt="L'onglet Bibliothèque sur un téléphone : les pistes de la catégorie Music, chacune avec une case à cocher et un casque pour la pré-écoute, et en dessous la carte de pré-écoute avec la forme d'onde de la piste et ses marqueurs">
</p>

- *Direct :* chaque lecteur sous forme de carte, et le mur de cartouches.
  Chaque carte a, de haut en bas, le nom du lecteur, son état, Single/Cont et
  CUE ; la pochette, le titre et l'artiste et la piste suivante ; la forme
  d'onde (un appui déplace la lecture) ; puis le temps écoulé et la durée de la
  piste avec le temps restant en grands chiffres au-dessus du transport
  (Lecture/Suivante, Précédente, Pause, Stop, Reprendre au début, Stop en
  fondu, Stop à la fin de la piste en cours) avec des boutons de la taille d'un
  doigt, et un fin vumètre avec le fader de volume (le niveau du fader
  s'affiche en dessous). Le temps restant se termine là où les boutons se
  terminent, plus petit pour une longue piste. Sur la plupart des téléphones,
  le vumètre montre son échelle à sa gauche, et sur un téléphone plus large
  (environ 406 dp et plus) le fader montre aussi ses niveaux ; sur un
  téléphone étroit (moins d'environ 384 dp), ni l'un ni l'autre, et les
  boutons rétrécissent un peu pour tenir. Les boutons fonctionnent pendant
  qu'un autre doigt tient le fader, et un appui compte même si le doigt roule
  un peu ou reste posé un instant. Tenu à la verticale, les cartes sont en haut,
  une carte entière et le début de la suivante (faites défiler pour en voir
  plus), et le mur de cartouches en dessous. Tourné sur le côté, chaque lecteur
  est une rangée à gauche (son nom et son état, le titre, le temps restant,
  Lecture/Suivante, Pause et Stop ; appuyez ailleurs sur la rangée pour ouvrir
  le lecteur dans Playlists) et le mur de cartouches est à droite. Appuyez sur
  **CARTOUCHES** dans son en-tête pour le replier : seul l'en-tête reste, en
  bas (les onglets de page et *Tout arrêter* fonctionnent toujours), et les
  lecteurs prennent la place ; appuyez de nouveau pour le déplier. L'onglet
  Cartouches montre toujours le mur de cartouches entier. Une page de
  cartouches trop large pour la place s'affiche sur moins de colonnes, dans le
  même ordre.
- *Cartouches :* la page de cartouches affichée, remplissant l'écran. Balayez
  latéralement pour afficher la page suivante ou la précédente. Tenue à la
  verticale, une page plus large que haute est tournée (8 × 2 s'affiche en
  2 × 8), dans le même ordre.
- *Playlists :* les pastilles en haut (P1, P2…) choisissent le lecteur ; en
  dessous se trouvent ses onglets de playlist (+ en crée une nouvelle), la
  liste des pistes et, en bas (à droite quand le téléphone est sur le côté), la
  carte du lecteur avec une forme d'onde plus grande pour préparer une piste
  (elle raccourcit quand la place manque, si bien que les boutons restent
  toujours visibles). Un appui long sur une ligne ouvre son menu, qui a aussi
  **Monter**, **Descendre** et **Déplacer vers un autre lecteur** (la piste va
  juste après la piste suivante de ce lecteur, dans la playlist qu'il affiche ;
  la piste à l'antenne reste). Faites glisser une ligne par sa poignée ≡ pour
  la déplacer. Le pied de page a **Ajouter** et, derrière **⋯**,
  **Réinitialiser**. Un appui long sur la forme d'onde place un marqueur à cet
  endroit (cue-in, fin d'intro, début d'outro, MIX, cue-out) ou en retire un
  que vous avez placé ; les marqueurs se modifient ici seulement, pas dans
  Direct.
- *Bibliothèque :* la bibliothèque de pistes et la musique de l'appareil (voir
  [Bibliothèque](#library)).

Retour depuis un autre écran mène à Direct ; sur Bibliothèque, il revient
d'abord à la liste des catégories (sur la musique de l'appareil, d'abord d'un
dossier vers le haut).

Quand rien ne bouge à l'écran, l'application le dessine une fois par seconde
(pour l'horloge) afin d'économiser la batterie ; un vumètre qui redescend
encore après une pause ou un arrêt la garde en train de dessiner jusqu'à ce
qu'il soit retombé.

**Le cadenas** protège des appuis parasites pendant une émission. Verrouillés
(le cadenas fermé et allumé), les faders de volume ne font que montrer le
niveau et les formes d'onde ne déplacent plus la lecture ni ne modifient les
marqueurs ; ils paraissent atténués. Un pincement et un glissement zooment et
déplacent toujours une forme d'onde, et Lecture, le transport, les cartouches
et les playlists fonctionnent toujours. Le verrouillage reste après un
redémarrage et est enregistré avec la session, si bien que charger une autre
session reprend le verrouillage de cette session ; l'API à distance et le MIDI
ne sont pas verrouillés.

Un glissement qui commence sur un fader de volume le déplace ; un qui commence
ailleurs fait défiler, si bien que faire défiler ne change jamais un niveau à
l'antenne. Un appui sur une forme d'onde déplace la lecture à cet endroit, un
pincement à deux doigts dessus zoome autour de vos doigts, un glissement à un
doigt déplace une vue zoomée et **Vue complète** montre de nouveau toute la
piste. Dans une playlist, un doigt fait défiler ; pour déplacer une piste,
utilisez sa poignée ≡ ou le menu de la ligne.

Sur une **tablette**, la disposition est celle de l'ordinateur (voir
[Disposition](layout.md)), avec des cibles plus faciles à atteindre.

Un appui long ouvre ce qu'ouvre un clic droit sur l'ordinateur : le menu d'une
ligne, les menus d'en-tête, le menu d'une cartouche (**Modifier…** choisit le
fichier d'une cartouche vide). Sur un téléphone, les Paramètres remplissent
l'écran et listent leurs sections ; appuyez sur l'une pour l'ouvrir, avec le
nom de chaque réglage au-dessus de sa commande. La flèche ou Retour revient à
la liste, et Retour de nouveau (ou **Fermer**) ferme les Paramètres. Modifier
une cartouche ouvre directement sa section.

À propos → **Licences tierces** ouvre les mentions des bibliothèques avec
lesquelles l'application est construite dans un navigateur ou un lecteur HTML.

## Bibliothèque {#library}

Sur un **téléphone**, l'onglet **Bibliothèque** s'ouvre sur une liste de
catégories : **Tous les titres**, vos catégories (chaque sous-catégorie sous
son parent, avec sa couleur et le nombre de pistes qu'elle contient), **Sans
catégorie** et **Musique de l'appareil**. La bibliothèque est la même que
celle que montre la [fenêtre de la bibliothèque](library.md) de l'ordinateur :
chaque fichier ajouté à une playlist, à une cartouche ou à la bibliothèque s'y
trouve. Les catégories se créent, se renomment, se recolorent et se suppriment
dans **Paramètres → Bibliothèque**.

**Les pistes d'une catégorie.** Appuyez sur une catégorie pour ouvrir ses
pistes (avec celles de ses sous-catégories). La flèche dans son en-tête, ou
Retour, revient aux catégories. Un autre onglet puis le retour retrouve la
liste comme vous l'avez laissée. Ouvrir une catégorie depuis les catégories
commence avec une recherche vide. La recherche filtre la liste par titre,
artiste, album et les autres tags que cherche la fenêtre de la bibliothèque ;
**Toute la bibliothèque** cherche dans toutes les catégories. Chaque ligne
montre le titre (ou le nom du fichier), l'artiste, la durée et les marques de
la fenêtre pour un fichier manquant, illisible ou pas encore analysé.

Appuyez sur des lignes pour les cocher. Tant que quelque chose est coché, la
barre de la musique de l'appareil apparaît en bas : combien, le lecteur et la
playlist où elles vont (appuyez dessus pour en choisir une autre), **Ajouter à
la fin** et **Ajouter après la suivante**, comme sur la musique de l'appareil.
Les pistes vont dans l'ordre où vous les avez cochées, et l'ajout efface les
coches. Revenir aux catégories les efface aussi.

**Le menu d'une ligne.** Un appui long sur une ligne ouvre son menu : **Ajouter
à ▸** (la playlist d'un lecteur, à la fin), **Pré-écouter**, **Déplacer vers la
catégorie ▸**, **Réanalyser** et **Retirer de la bibliothèque**, qui demande
comme sur l'ordinateur (voir [Retirer des pistes](library.md#removing-tracks)).
Tant que des lignes sont cochées, un appui long sur une autre ligne la coche
aussi et le menu agit sur chaque ligne cochée, dans l'ordre où vous les avez
cochées ; **Pré-écouter** n'est alors pas proposé. Les coches restent après
l'action du menu.

**Pré-écoute.** Le 🎧 à droite d'une ligne joue sa piste depuis son cue-in sur
la sortie de pré-écoute de la bibliothèque ; le 🎧 d'une autre ligne passe à
cette piste. Un fin bandeau au-dessus de la liste montre le titre, le temps
restant, **Pause**/**Lecture** et **Stop** pendant qu'elle joue (ou est en
pause), le titre grisé après **Stop**, et une indication avant la première
pré-écoute : il garde toujours sa place, si bien que les lignes ne bougent pas
quand une pré-écoute démarre ou s'arrête. Le 🎧 est grisé quand le fichier est
manquant ou ne peut pas être lu.

**La carte de pré-écoute.** Touchez le bandeau (hors de ses boutons) pour
ouvrir la carte sous la liste, avec la forme d'onde de la piste comme sur la
carte d'un lecteur dans Playlists ; la flèche à droite du bandeau indique si
elle est ouverte. Touchez de nouveau le bandeau, ou appuyez sur Retour, pour
la fermer ; la pré-écoute continue. La flèche de l'en-tête de la liste
revient aussitôt aux catégories et laisse la carte ouverte pour la liste
suivante. Sur la carte :

- un toucher sur la forme d'onde saute à cet endroit ou, à l'arrêt, fixe
  où **Pré-écouter** commence ;
- un appui long place un marqueur à cet endroit (cue-in, fin d'intro,
  début d'outro, MIX, cue-out) ou en retire un que vous avez placé. Les
  marqueurs appartiennent à la piste : la modification se voit dans toutes
  les playlists et sur l'ordinateur ;
- un pincement zoome, un glissement déplace une vue zoomée et **Vue
  complète** montre de nouveau toute la piste ;
- sous la forme d'onde se trouvent les temps, **Pré-écouter**, **Pause**,
  **Stop** et l'indicateur de niveau.

La liste reste au-dessus de la carte (à gauche, téléphone à l'horizontale),
si bien que le 🎧 d'une autre ligne passe toujours la carte à cette piste. Le
cadenas arrête les sauts et les modifications de marqueurs de la forme
d'onde, comme sur les lecteurs.
La pré-écoute
s'arrête quand sa piste quitte la bibliothèque et quand vous quittez l'onglet
Bibliothèque (pour un autre onglet, ou par Retour) ; sur les catégories ou sur
la musique de l'appareil, elle continue, mais son bandeau ne s'affiche que sur
la liste des pistes.

Contrairement à l'ordinateur, Android (un téléphone ou une tablette) joue la
pré-écoute de la bibliothèque sur la sortie par défaut quand aucune sortie
n'est choisie pour elle : dans **Paramètres → Sorties audio → Bibliothèque**, le
choix vide se lit *Par défaut du système*, et son **Tester Cue** joue là.
Choisissez-y une autre sortie (un casque filaire, par exemple) pour pré-écouter
à l'écart de l'antenne. La sortie par défaut est en général aussi celle des
lecteurs et du mur de cartouches, si bien que la pré-écoute peut être entendue
à l'antenne : tant que quelque chose joue sur cette sortie, le 🎧 demande
d'abord (**Pré-écouter quand même** ou **Annuler**), et une pré-écoute que vous
n'avez pas confirmée s'arrête quand quelque chose démarre là, comme le CUE d'un
lecteur (voir [CUE (pré-écoute)](players.md#cue-pre-listen)).

**Ajouter de la musique de l'appareil à une catégorie.** Le **＋** dans
l'en-tête d'une liste de pistes ouvre la musique de l'appareil pour cette
catégorie. Sa barre n'a alors que **Ajouter à &lt;catégorie&gt;** (**Ajouter à Sans
catégorie** depuis *Tous les titres* ou *Sans catégorie*), qui ajoute les
fichiers cochés à la bibliothèque dans cette catégorie et revient à la liste ;
un fichier déjà dans la bibliothèque la rejoint s'il n'a pas de catégorie et
garde celle qu'il a sinon, comme sur l'ordinateur. La musique de l'appareil
garde ses coches entre les visites, si bien que les fichiers cochés lors d'une
visite antérieure sont ajoutés aussi : vérifiez d'abord le nombre sur la barre.
**Retour à la liste**, en haut, ou Retour revient à la liste sans rien ajouter.
La musique de l'appareil ouverte depuis les catégories a plutôt **Retour aux
catégories** en haut.

**Le dossier d'appareil d'une catégorie.** Une catégorie peut prendre ses
fichiers dans un dossier de la musique de l'appareil. Dans son menu (un appui
long sur elle dans **Paramètres → Bibliothèque**, ou dans la fenêtre de la
bibliothèque sur une tablette), **Utiliser un dossier de l’appareil…** ouvre la
musique de l'appareil pour choisir le dossier (**Utiliser ce dossier**) ; le
menu de la catégorie le nomme ensuite, au-dessus de ses éléments de dossier.
Rien ne surveille le dossier : quand vous y avez copié de la nouvelle musique,
appuyez sur le **↻** de l'en-tête de la liste des pistes de la catégorie (ou
sur **Relire le dossier de l’appareil** dans son menu). Chaque fichier que la
musique de l'appareil liste dans le dossier et ses sous-dossiers est ajouté à
la catégorie, avec les règles ci-dessus (un fichier déjà dans une autre
catégorie la garde), et un avis dit combien de fichiers ont été lus. Les
fichiers qui ont quitté le dossier restent dans la bibliothèque, marqués manquants, et redeviennent normaux quand ils y
reviennent (quand la musique de l'appareil ne liste rien, par exemple sans
autorisation, rien n'est marqué). **Changer le dossier de l’appareil…** et **Oublier le dossier de l’appareil**
sont dans le même menu ; un dossier appartient à une seule catégorie.

### Musique de l'appareil {#device-music}

**Musique de l'appareil** montre les fichiers audio de l'appareil par
**Dossiers** (avec le chemin au-dessus de la liste ; appuyez sur une partie
pour remonter), **Artistes** ou **Albums**, et une recherche sur le titre,
l'artiste, l'album et le nom de fichier. Chaque ligne montre le titre, l'artiste
et la durée ; appuyez sur des lignes pour les cocher. Tant que quelque chose est
coché, une barre en bas montre combien, le lecteur et la playlist où elles vont
(appuyez dessus pour en choisir une autre), **Ajouter à la fin** et, quand la
piste suivante de ce lecteur est dans cette playlist, **Ajouter après la
suivante**. Les pistes vont dans l'ordre où vous les avez cochées. Un fichier de
l'appareil n'a pas de pré-écoute.

**Ajouter** dans le pied de page d'une playlist, le fichier d'une cartouche et
Paramètres → Playlists → Dossier de musique ouvrent la musique de l'appareil
sur un téléphone (sur une tablette, la fenêtre du **navigateur de musique**, par
dossier). Cochez des fichiers et appuyez sur **Ajouter**, appuyez sur un seul
fichier pour une cartouche, ou sur **Utiliser ce dossier** ; **Annuler** ou
Retour revient là où vous étiez. Copiez de la musique sur l'appareil (par
exemple dans son dossier *Musique*) pour la voir là. Fauste Player ne fait que
lire votre musique : les tags et les pochettes ne sont pas écrits, et les
playlists et les pages de cartouches ne sont pas encore importées ni exportées
sur Android.

Sur une **tablette**, la bibliothèque est la [fenêtre de la
bibliothèque](library.md) de l'ordinateur, ouverte depuis la barre supérieure.

## Sorties {#outputs}

Paramètres → Sorties audio liste les sorties du téléphone par type : le
haut-parleur, un casque filaire, une interface USB, une enceinte Bluetooth
(« JBL Flip (Bluetooth) »). Chaque lecteur, le mur de cartouches et leur CUE en
choisissent une, comme sur l'ordinateur, et la pré-écoute de la bibliothèque
aussi (voir [Bibliothèque](#library)). Une enceinte Bluetooth éteinte puis
rallumée est retrouvée par sa route ; tant qu'elle est absente, ses lecteurs se
comportent comme avec un périphérique manquant sur l'ordinateur. La possibilité
que deux sorties sonnent en même temps dépend du téléphone.

## Pas sur Android {#not-on-android}

Les surfaces de contrôle MIDI, le contrôle à distance, la modification des tags
et des pochettes, les fichiers de playlist et de pages de cartouches, et
l'automatisation (le remplissage musical et Paramètres → Automatisation) sont
pour l'instant des fonctions de l'ordinateur. Un clavier branché sur une
tablette fonctionne comme sur l'ordinateur (voir [Clavier](keyboard.md)).
