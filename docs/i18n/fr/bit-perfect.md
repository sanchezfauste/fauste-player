# Sortie bit-perfect

Un périphérique **bit-perfect** reçoit les échantillons de chaque fichier
exactement tels qu'ils sont dans le fichier : même fréquence d'échantillonnage,
mêmes valeurs, sans rééchantillonnage, changement de volume ni mixage.

La sortie bit-perfect est un **mode particulier pour l'écoute audiophile** :
écouter une archive, alimenter une chaîne hi-fi ou de monitoring, vérifier une
liaison numérique, partout où tout traitement sur l'ordinateur doit être évité.
Elle est désactivée par défaut et n'est pas faite pour un programme musical
mixé. Lisez [Ce que cela coûte](#what-it-costs) avant de l'activer.

## Ce que cela coûte {#what-it-costs}

Pour garder chaque piste intacte, un lecteur dont la sortie Main est sur un
périphérique bit-perfect joue autrement que les autres :

- **Ni fondus enchaînés ni segues.** Chaque piste est lue jusqu'à sa fin (son
  cue-out, ou la fin du fichier avec **Utiliser le cue-in et le cue-out**
  désactivé), puis la suivante démarre. **Enchaînement automatique au point
  MIX** et le point MIX ne sont pas utilisés : le marqueur MIX est dessiné
  atténué et le badge **Mix** ne s'allume jamais. **PLAY / SUIV.** et
  **Précédente** pendant qu'une piste est à l'antenne basculent aussitôt, avec
  un court arrêt sans clic, au lieu de faire un fondu.
- **Un court silence quand la fréquence d'échantillonnage change.** Entre deux
  pistes dont les fréquences diffèrent, quand rien d'autre ne sonne sur le
  périphérique, le périphérique joue d'abord ce qu'il a encore en tampon (la
  fin de la piste), puis est rouvert à la nouvelle fréquence, ce qui laisse un
  court silence (généralement quelques dizaines de millisecondes, plus long
  sur certains périphériques). Les pistes à la même fréquence s'enchaînent
  sans blanc ni chevauchement. Quand la fréquence ne peut pas suivre (voir
  [Ce qui se passe sur un périphérique bit-perfect](#what-happens-on-a-bit-perfect-device)),
  la piste suivante démarre aussitôt, convertie à la fréquence du périphérique.
- **Pas de normalisation du loudness.** Une piste sur un périphérique
  bit-perfect joue toujours au gain unitaire, même avec la
  [normalisation du loudness](loudness.md#bit-perfect-outputs) activée :
  l'en-tête affiche **0.0 dB** en ambre, et son infobulle en dit la raison. Les
  autres lecteurs et cartouches sont normalisés comme d'habitude.
- **Le volume est maintenu à 100 %.** Le fader de volume du lecteur est
  verrouillé en haut : à l'écran, sur les contrôleurs [MIDI](midi.md) et par
  l'[API à distance et l'OSC](remote-control.md). Il passe à 100 % quand le
  bit-perfect est activé pour le périphérique, et y reste quand il est
  désactivé.
- **Ce qui fonctionne toujours :** **Stop en fondu** (il fait un fondu sortant
  sur la durée du Stop en fondu), **Pause**, **Stop**, **Reprendre au début**, les
  marques de répétition et d'arrêt après, les modes Single et Continu, le Cue
  (la pré-écoute, qui n'a pas de fader et n'est pas affectée), les
  cartouches, la [sortie DSD](#dsd), et chaque lecteur dont la sortie Main est
  sur un autre périphérique, qui mixe comme d'habitude. Une cartouche peut
  jouer sur le même périphérique : pendant qu'elle sonne, la piste suivante du
  lecteur démarre à la fréquence actuelle du périphérique, convertie (elle
  n'est donc pas bit-perfect), et aucun silence n'est inséré.
- **Quand l'utiliser.** Activez-le quand chaque piste doit parvenir au
  convertisseur intacte. Laissez-le désactivé pour un programme à l'antenne
  qui a besoin de transitions mixées ou de changements de niveau. Le mode se
  règle par périphérique, si bien que les lecteurs mixés peuvent rester sur
  un autre périphérique.

## Rendre un périphérique bit-perfect {#setting-a-device-bit-perfect}

1. Dans **Paramètres → Sorties audio**, choisissez explicitement le
   périphérique pour la sortie Main d'un lecteur (ou celle du mur de
   cartouches). Un lecteur laissé sur le périphérique par défaut du système ne
   peut pas être rendu bit-perfect. Un périphérique qu'aucune sortie n'utilise
   plus perd son interrupteur bit-perfect et son mode DSD au prochain
   démarrage de l'application.
2. Choisissez **Avancé** en haut de la section. Sous **Réglages par
   périphérique**, activez **Bit-perfect** à côté du périphérique.
3. Le périphérique devient bit-perfect dès que rien n'y joue (voir
   [Réglages en attente](settings.md#settings-pending)).

Le périphérique démarre alors à sa propre fréquence d'échantillonnage si vous
lui en avez donné une au même endroit, sinon à la fréquence d'échantillonnage
globale, et suit chaque fichier à partir de là.

L'interrupteur est désactivé quand le périphérique ne peut pas donner
l'accès exclusif.
- **Linux :** choisissez un périphérique ALSA dont le nom commence par `hw:`.
  C'est la carte son elle-même. PulseAudio, PipeWire, JACK et les
  périphériques ALSA `default` ou `plughw:` mixent ou convertissent, ils ne
  sont donc jamais bit-perfect.
- **Windows :** choisissez le périphérique du système **WASAPI**. Il est ouvert
  en mode exclusif.
  - Dans les réglages sonores de Windows, les propriétés **Avancées** du
    périphérique doivent avoir *Autoriser les applications à prendre le
    contrôle exclusif de ce périphérique* activé (c'est le cas par défaut).
  - Pendant qu'il joue, aucun autre programme ne peut utiliser le
    périphérique.
- **macOS :** choisissez le périphérique sur **Core Audio**. Il est ouvert en
  mode hog.
  - La fréquence d'échantillonnage du périphérique est réglée sur celle de la
    piste, et son format sur le format entier le plus large qu'il offre à
    cette fréquence (les réglages que montre Configuration audio et MIDI).
  - Ils sont rendus quand l'application cesse d'utiliser le périphérique.
  - Deux périphériques portant exactement le même nom ne peuvent pas être
    rendus bit-perfect.

## Ce qui se passe sur un périphérique bit-perfect {#what-happens-on-a-bit-perfect-device}

- **Accès exclusif.** Rien d'autre sur l'ordinateur ne peut jouer sur le
  périphérique tant que l'application l'utilise. Si l'accès exclusif est
  refusé, le périphérique joue quand même, en partagé, et le badge BP reste
  éteint.
- **La fréquence suit le fichier.** Le périphérique est rouvert à la fréquence
  d'échantillonnage d'une piste quand la piste démarre :
  - **Depuis le silence :** vous lisez une piste, en reprenez une chargée en
    pause, pré-écoutez ou lancez une cartouche pendant que rien ne joue sur le
    périphérique. Les pistes qui ne font qu'attendre (la piste suivante de
    chaque lecteur) sont préparées de nouveau à la nouvelle fréquence.
  - **Entre deux pistes d'un lecteur :** la piste à l'antenne s'arrête (à sa
    fin, ou aussitôt pour **PLAY / SUIV.** et **Précédente**), le périphérique
    se rouvre et la piste suivante démarre. C'est le court silence de
    [Ce que cela coûte](#what-it-costs).
  - Le périphérique n'est rouvert qu'une fois qu'il a joué tout ce qu'il avait
    en tampon, si bien que la fin de la piste précédente n'est jamais coupée.
    Ensuite, la réouverture prend le temps dont le périphérique a besoin pour
    démarrer. Ensemble, cela fait généralement quelques dizaines de
    millisecondes ; le démarrage a autant de retard. L'attente du tampon n'est
    jamais plus longue d'une demi-seconde (`tuning.play_out_max_ms` dans le
    fichier de configuration, de 0 à 2000 ms), quoi que signale le
    périphérique.
- **Quand la fréquence ne peut pas suivre.** Le périphérique garde sa
  fréquence, et une piste à une autre fréquence est convertie sur toute sa
  durée (elle n'est pas bit-perfect), quand :
  - autre chose sonne sur le périphérique au démarrage de la piste : un autre
    lecteur (ou un lecteur en pause avec une piste), une cartouche ou une
    pré-écoute ;
  - le périphérique a déjà refusé cette fréquence ; il garde la précédente ;
  - la piste n'a pas encore été analysée, donc sa fréquence n'est pas connue.

  Le passage d'une piste à l'autre se fait quand même sans fondu enchaîné.
- **Aucun traitement, quand rien ne le demande.** Les échantillons passent
  intacts tant qu'aucun fondu n'est en cours (Stop en fondu, les rampes de
  pause et de reprise, le court arrêt sans clic d'une piste) et que rien
  d'autre ne joue sur les mêmes sorties (un autre lecteur, une cartouche, une
  tonalité de test). Le volume n'est pas un problème : une sortie bit-perfect
  le maintient à 100 %.

## DSD {#dsd}

Un fichier DSD est normalement lu converti en PCM, comme n'importe quel autre
fichier. Un périphérique bit-perfect peut au contraire recevoir le flux DSD
intact.

**Les trois modes.** Dans la vue Avancé de Paramètres → Sorties audio, chaque
périphérique qu'utilise une sortie a un choix **DSD** sous son interrupteur
bit-perfect :
- **Convertir en PCM** (par défaut) : le DSD est converti, comme sur tout
  autre périphérique.
- **DoP** (DSD over PCM) : les bits DSD voyagent à l'intérieur
  d'échantillons PCM 24 bits, que la plupart des convertisseurs compatibles
  DSD reconnaissent. Il fonctionne sur tous les systèmes.
- **DSD natif** (Linux uniquement) : DSD brut, pour les périphériques ALSA
  `hw:` dont le pilote annonce un format d'échantillons DSD.

Seuls les modes que le périphérique peut accepter sont proposés, et une ligne
sous le choix dit pourquoi les autres ne le sont pas : le périphérique n'est
pas branché, le bit-perfect est désactivé, le périphérique ne peut pas être
ouvert en exclusif, le DSD natif nécessite Linux, ou le périphérique n'accepte
pas le DSD natif. Un mode enregistré pour un périphérique qui ne peut plus
l'accepter s'affiche comme PCM, ce qui est ce qui joue ; le mode enregistré
revient quand le périphérique peut de nouveau l'accepter. Changer un mode, le
réglage de mixage ou le silence DSD s'applique dès que rien ne joue sur le
périphérique, comme les autres réglages de sortie.

**Quand le DSD sort intact.** Toutes ces conditions doivent être réunies au
démarrage de la piste :
- le périphérique est bit-perfect, avec accès exclusif, et son mode est DoP ou
  DSD natif ;
- la piste est en DSD (DSF ou DFF), mono ou stéréo, et a été analysée (c'est
  ainsi que sa fréquence DSD est connue) ;
- rien d'autre ne joue sur le périphérique (un autre lecteur, une cartouche,
  une tonalité de test) ;
- la piste n'a aucun gain de [normalisation du loudness](loudness.md#bit-perfect-outputs) :
  une piste chargée (par exemple en pause) avec un gain avant que le
  périphérique devienne bit-perfect le garde, si bien qu'elle est lue
  convertie, sans **DSD** ni **BP** ;
- le périphérique accepte le flux. Le DoP exige une fréquence de périphérique
  égale à la fréquence DSD divisée par 16 (176,4 kHz pour le DSD64, 352,8 kHz
  pour le DSD128, 705,6 kHz pour le DSD256) et un format 24 ou 32 bits. Le DSD
  natif exige un périphérique qui accepte le format DSD à cette fréquence.

Quand le DSD natif prend fin, le périphérique repasse en PCM à la fréquence
qu'il avait avant la piste DSD, parce que beaucoup de convertisseurs acceptent
le DSD natif à des fréquences qu'ils ne peuvent pas jouer en PCM (aucun
convertisseur ne joue de PCM à la fréquence à laquelle tourne le DSD512). Une
piste qui se poursuit en PCM garde la fréquence du flux DSD quand le
périphérique l'accepte en PCM, et sinon continue à la fréquence antérieure
depuis l'endroit où elle en était, comme tout ce qui joue sur ce périphérique ;
un fondu en cours s'y termine aussitôt. Le périphérique ne reste jamais sur une
fréquence qu'il refuse : si aucune fréquence ne s'ouvre (par exemple, le
périphérique a été débranché à ce moment-là), la sortie est perdue jusqu'à ce
que la nouvelle tentative automatique la rouvre à la fréquence antérieure.

Sinon, la piste est convertie en PCM et le journal en donne la raison (par
exemple « something else plays on the device » ou « the device refused 705600
Hz »). La pré-écoute et les cartouches sont toujours converties.

Tant que le DSD sort intact :
- le badge de l'en-tête affiche **DSD** au lieu de **BP** ;
- les vumètres montrent le niveau de la conversion PCM de la même piste, ils
  fonctionnent donc comme d'habitude ;
- Stop et un stop en fondu arrêtent la piste aussitôt, sans fondu, puisqu'un
  flux DSD ne peut pas être estompé ;
- la pause et la reprise agissent aussi aussitôt, sans rampe.

**Silence aux extrémités.** Chaque démarrage, fin et passage en PCM envoie
d'abord du silence DSD (200 ms par défaut), pour que le convertisseur se
verrouille sans clic. L'exception est une piste DSD qui poursuit un flux de
même nature et de même fréquence DSD dont le silence est encore en cours : le
convertisseur est encore verrouillé, elle démarre donc sans silence
supplémentaire. Une piste démarre donc avec ce retard, et un passage en PCM
laisse un blanc de cette durée (un périphérique DSD natif joue aussi ce qu'il a
en tampon avant d'être rouvert en PCM, quelques millisecondes de plus). C'est **Silence DSD** dans Paramètres →
Sorties audio, Avancé (0 à 2000 ms).

**Quand une autre source a besoin du périphérique.** **Quand une autre source
a besoin d'une sortie DSD**, dans Paramètres → Sorties audio, choisit ce qui
se passe quand un autre lecteur, une cartouche ou une tonalité de test démarre
sur le même périphérique :
- **Poursuivre la piste DSD en PCM** (par défaut). Le flux passe en PCM après
  le silence DSD, et la piste continue, convertie, depuis l'endroit où elle en
  était. La même chose arrive à la piste qui suit d'elle-même (voir plus bas).
- **Garder le DSD et couper les autres sources.** Rien n'interrompt le flux
  DSD. Les autres sources routées vers le périphérique sont coupées jusqu'à la
  fin de la piste DSD, et le lecteur affiche entre-temps un badge **Autres
  coupées**. La piste suivante propre au lecteur
  démarre à la fin de la piste DSD : une piste PCM après le silence DSD, une
  piste DSD de même nature et de même fréquence DSD aussitôt, en poursuivant
  le flux.

**Un album ne reste pas en DSD avec le réglage par défaut.** Avec *Poursuivre
la piste DSD en PCM*, seule une piste DSD qui démarre sur un périphérique
inactif sort en DSD. Les pistes que le lecteur démarre ensuite d'elles-mêmes
(à la fin d'une piste)
démarrent à partir d'un préchargement, qui est toujours en PCM, si bien que
le périphérique passe en PCM et qu'elles sont lues converties. Une piste que
vous démarrez vous-même (PLAY, double-clic) sort de nouveau en DSD quand le
périphérique est inactif ou que le flux DSD précédent est encore dans son
silence à la même fréquence DSD. Pour garder tout un album DSD en DSD,
choisissez *Garder le DSD et couper les autres sources*. Chaque piste du
lecteur sort alors en DSD, et la suivante démarre quand la précédente se
termine.

Si le périphérique est perdu pendant que le DSD joue et revient incapable de
le porter (par exemple sans accès exclusif), la piste continue en PCM.

## Le badge BP {#the-bp-badge}

Le badge **BP** dans l'en-tête du lecteur n'existe que sur un lecteur dont le
périphérique Main fonctionne en bit-perfect. Il
s'allume (bordure et texte verts ; gris atténué quand il est éteint) tant que la piste en cours parvient intacte à son périphérique Main.
Toutes ces conditions doivent être réunies :

- le périphérique est bit-perfect et ouvert avec accès exclusif ;
- le périphérique tourne à la fréquence d'échantillonnage de la piste ;
- la piste est en PCM entier sans perte (WAV, AIFF, FLAC, ALAC, WavPack ou
  Monkey's Audio), mono ou stéréo, et d'au plus 24 bits, et le format du
  périphérique contient sa taille d'échantillon (un fichier 24 bits sur un
  périphérique 16 bits n'est pas bit-perfect). Le DSD est converti, il
  n'allume donc jamais BP ; quand il sort intact (voir [DSD](#dsd)), le badge
  affiche **DSD** à la place ;
- la piste a été analysée, puisque c'est ainsi que sa fréquence et sa taille
  d'échantillon sont connues. Les pistes qu'une version antérieure a analysées
  reçoivent leur format une fois analysées de nouveau (l'avis de démarrage, ou
  Paramètres → Analyse), ou dès qu'un lecteur les affiche ou qu'une cartouche
  les contient ;
- la piste joue au gain unitaire : une sortie bit-perfect n'est jamais
  normalisée, mais une piste qui a démarré avec un gain de normalisation avant
  que son périphérique devienne bit-perfect garde ce gain jusqu'à son arrêt,
  et BP reste éteint pour elle ;
- le volume est maintenu à 100 % sur une sortie bit-perfect, aucun fondu n'est
  en cours (Stop en fondu, les rampes de pause et de reprise et le court arrêt
  sans clic d'une piste sont les seuls fondus qui restent sur un tel lecteur),
  et rien d'autre ne joue sur les mêmes sorties.

Certains fichiers ne sont jamais affichés comme bit-perfect :
- **Les fichiers avec perte** (MP3, AAC, Ogg Vorbis, Opus) : leurs échantillons
  décodés ne sont pas les valeurs entières qu'accepte un périphérique.
- **Les fichiers de plus de 24 bits :** le mixeur travaille en virgule
  flottante 32 bits, qui porte 24 bits exactement.
- **Les fichiers de plus de deux canaux :** ils sont mixés en stéréo.

## Le vérifier soi-même {#checking-it-yourself}

Pour vérifier une chaîne de bout en bout :

1. Connectez la sortie numérique du périphérique (S/PDIF, AES ou boucle USB) à
   un enregistreur qui capture à l'identique, bit pour bit.
2. Jouez un fichier de test sans perte sans que rien d'autre ne joue.
3. Enregistrez-le.
4. Comparez l'enregistrement avec le fichier. Par exemple, avec SoX, inversez
   l'un et mixez-les : `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`
   après avoir aligné leurs débuts. Chaque échantillon de la différence doit
   être nul.
5. Vérifiez un changement de fréquence : en mode Continu, avec
   **Enchaînement automatique au point MIX** activé, jouez une piste à
   44,1 kHz qui a un point MIX, suivie d'une piste à 96 kHz. La première est
   lue jusqu'à son cue-out sans fondu ; après un court silence, le
   convertisseur affiche 96 kHz pour la seconde. Le badge BP s'allume sur les
   deux.
6. Jouez ensuite deux pistes à la même fréquence : la seconde suit la première
   sans blanc ni chevauchement.

Les tests automatisés du projet vérifient la même propriété à l'intérieur de
l'application, sur un périphérique simulé.

### Le DSD sur un vrai convertisseur {#dsd-on-a-real-converter}

Les tests automatisés ne vérifient le DSD que sur des périphériques simulés.
Le DoP et le DSD natif n'ont pas été essayés par le projet sur un vrai
convertisseur. Pour en vérifier un :
1. Réglez le périphérique sur **DoP** (ou **DSD natif** sous Linux) et jouez
   un fichier DSD sans que rien d'autre ne joue. L'en-tête doit afficher **DSD**, et l'affichage propre du convertisseur doit
   montrer la fréquence DSD (par exemple DSD64) au lieu d'une fréquence PCM. Un
   convertisseur qui affiche une fréquence PCM ou joue du bruit ne reconnaît
   pas le flux : revenez à **Convertir en PCM**.
2. Écoutez s'il y a un clic ou une rafale de bruit au démarrage, à l'arrêt, à
   la fin de la piste. Un clic signifie que le
   convertisseur a besoin d'un **Silence DSD** plus long (Paramètres → Sorties
   audio, Avancé).
3. Lancez une cartouche ou un autre lecteur sur le même périphérique, une fois
   avec chaque réglage de mixage, et vérifiez le comportement décrit plus
   haut.
4. Sous Linux, pour vérifier le DSD natif sans l'application, exécutez
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test it -- --ignored conformance::native_dsd_on_a_real_device`.
   Il ouvre le périphérique en DSD natif au DSD64 et joue une seconde de
   silence DSD. Il doit réussir, et le convertisseur doit se verrouiller sur le
   DSD64.
5. Avec des fichiers DSD dans `test-music/`, `cargo test --release -p fp-engine --test it dsd_real_music:: -- --ignored`
   les joue à travers le moteur sur un périphérique simulé et compare les mots
   avec les octets du fichier (voir [Tests](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
