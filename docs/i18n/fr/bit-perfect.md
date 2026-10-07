# Sortie bit-perfect

Un périphérique **bit-perfect** reçoit les échantillons de chaque fichier
exactement tels qu'ils sont dans le fichier : même fréquence d'échantillonnage,
mêmes valeurs, sans rééchantillonnage, changement de volume ni mixage. C'est
utile pour les chaînes de monitoring et les liaisons numériques, où tout
traitement sur l'ordinateur doit être évité.

## Rendre un périphérique bit-perfect {#setting-a-device-bit-perfect}

1. Dans **Paramètres → Sorties audio**, choisissez explicitement le
   périphérique pour la sortie Main d'un lecteur (ou celle du mur de
   cartouches). Un lecteur laissé sur le périphérique par défaut du système ne
   peut pas être rendu bit-perfect. Un périphérique qu'aucune sortie n'utilise
   plus perd son interrupteur bit-perfect et son mode DSD au prochain
   démarrage de l'application.
2. Choisissez **Avancé** en haut de la section. Sous **Réglages par
   périphérique**, activez **Bit-perfect** à côté du périphérique.
3. Redémarrez l'application.

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
- **La fréquence suit le fichier.** Quand rien ne joue sur le périphérique et
  qu'une piste à une autre fréquence d'échantillonnage démarre, le périphérique
  est rouvert à cette fréquence.
  - Cela se produit quand vous lisez une piste, en reprenez une chargée en
    pause, pré-écoutez ou lancez une cartouche. Les pistes qui ne font
    qu'attendre (la piste suivante de chaque lecteur) sont préparées de nouveau
    à la nouvelle fréquence.
  - La réouverture prend le temps dont le périphérique a besoin pour démarrer
    (généralement quelques dizaines de millisecondes). Le démarrage a autant de
    retard.
  - Tant que quelque chose joue sur le périphérique, la fréquence ne change
    jamais. Une piste à une autre fréquence qui démarre alors (par exemple une
    piste à 48 kHz mixée après une piste à 44,1 kHz, ou une piste démarrée
    pendant qu'un autre lecteur ou une cartouche joue sur le même périphérique)
    est convertie sur toute sa durée, et n'est pas bit-perfect.
  - Si le périphérique refuse une fréquence, il garde la précédente et la
    piste est convertie.
- **Aucun traitement, quand rien ne le demande.** Les échantillons passent
  intacts tant que toutes ces conditions sont réunies :
  - le volume du lecteur est à 100 % ;
  - aucun fondu n'est en cours ;
  - rien d'autre ne joue sur les mêmes sorties (un autre lecteur, une
    cartouche, une tonalité de test).

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
réglage de mixage ou le silence DSD nécessite un redémarrage, comme les autres
réglages de sortie.

**Quand le DSD sort intact.** Toutes ces conditions doivent être réunies au
démarrage de la piste :
- le périphérique est bit-perfect, avec accès exclusif, et son mode est DoP ou
  DSD natif ;
- la piste est en DSD (DSF ou DFF), mono ou stéréo, et a été analysée (c'est
  ainsi que sa fréquence DSD est connue) ;
- le volume du lecteur est à 100 % ;
- rien d'autre ne joue sur le périphérique (un autre lecteur, une cartouche,
  une tonalité de test) ;
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
- le volume doit rester à 100 % : l'infobulle du fader le dit. Le bouger fait
  passer la piste en PCM (voir plus bas) ;
- Stop et un stop en fondu arrêtent la piste aussitôt, sans fondu, puisqu'un
  flux DSD ne peut pas être estompé. Appuyer sur PLAY sur une autre piste
  pendant qu'elle joue la coupe de la même façon au lieu de faire un fondu
  enchaîné ;
- la pause et la reprise agissent aussi aussitôt, sans rampe.

**Silence aux extrémités.** Chaque démarrage, fin et passage en PCM envoie
d'abord du silence DSD (200 ms par défaut), pour que le convertisseur se
verrouille sans clic. L'exception est une piste DSD qui poursuit un flux de
même nature et de même fréquence DSD dont le silence est encore en cours : le
convertisseur est encore verrouillé, elle démarre donc sans silence
supplémentaire. Une piste démarre donc avec ce retard, et un passage en PCM
laisse un blanc de cette durée. C'est **Silence DSD** dans Paramètres →
Sorties audio, Avancé (0 à 2000 ms).

**Quand une autre source a besoin du périphérique.** **Quand une autre source
a besoin d'une sortie DSD**, dans Paramètres → Sorties audio, choisit ce qui
se passe quand un autre lecteur, une cartouche ou une tonalité de test démarre
sur le même périphérique (bouger le fader propre au lecteur est l'exception :
cela fait toujours passer la piste en PCM) :
- **Poursuivre la piste DSD en PCM** (par défaut). Le flux passe en PCM après
  le silence DSD, et la piste continue, convertie, depuis l'endroit où elle en
  était. La même chose arrive à la piste qui suit d'elle-même (voir plus bas).
- **Garder le DSD et couper les autres sources.** Rien n'interrompt le flux
  DSD. Les autres sources routées vers le périphérique sont coupées jusqu'à la
  fin de la piste DSD, et le lecteur affiche entre-temps un badge **Autres
  coupées**. La piste suivante propre au lecteur ne chevauche pas : elle
  démarre à la fin de la piste DSD, sans fondu enchaîné ni enchaînement. Une
  piste PCM attend le silence DSD ; une piste DSD de même nature et de même
  fréquence DSD poursuit le flux sans lui. Bouger le fader fait toujours
  passer la piste en PCM.

**Un album ne reste pas en DSD avec le réglage par défaut.** Avec *Poursuivre
la piste DSD en PCM*, seule une piste DSD qui démarre sur un périphérique
inactif sort en DSD. Les pistes que le lecteur démarre ensuite d'elles-mêmes
(à la fin d'une piste, lors d'un enchaînement ou d'un fondu enchaîné)
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

Le badge **BP** dans l'en-tête du lecteur s'allume tant que la piste en cours
parvient intacte à son périphérique Main. Toutes ces conditions doivent être
réunies :

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
- le volume est à 100 %, aucun fondu n'est en cours, et rien d'autre ne joue
  sur les mêmes sorties.

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
2. Jouez un fichier de test sans perte à 100 % sans que rien d'autre ne joue.
3. Enregistrez-le.
4. Comparez l'enregistrement avec le fichier. Par exemple, avec SoX, inversez
   l'un et mixez-les : `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`
   après avoir aligné leurs débuts. Chaque échantillon de la différence doit
   être nul.

Les tests automatisés du projet vérifient la même propriété à l'intérieur de
l'application, sur un périphérique simulé.

### Le DSD sur un vrai convertisseur {#dsd-on-a-real-converter}

Les tests automatisés ne vérifient le DSD que sur des périphériques simulés.
Le DoP et le DSD natif n'ont pas été essayés par le projet sur un vrai
convertisseur. Pour en vérifier un :
1. Réglez le périphérique sur **DoP** (ou **DSD natif** sous Linux),
   redémarrez, et jouez un fichier DSD à 100 % sans que rien d'autre ne joue.
   L'en-tête doit afficher **DSD**, et l'affichage propre du convertisseur doit
   montrer la fréquence DSD (par exemple DSD64) au lieu d'une fréquence PCM. Un
   convertisseur qui affiche une fréquence PCM ou joue du bruit ne reconnaît
   pas le flux : revenez à **Convertir en PCM**.
2. Écoutez s'il y a un clic ou une rafale de bruit au démarrage, à l'arrêt, à
   la fin de la piste et en bougeant le fader. Un clic signifie que le
   convertisseur a besoin d'un **Silence DSD** plus long (Paramètres → Sorties
   audio, Avancé).
3. Lancez une cartouche ou un autre lecteur sur le même périphérique, une fois
   avec chaque réglage de mixage, et vérifiez le comportement décrit plus
   haut.
4. Sous Linux, pour vérifier le DSD natif sans l'application, exécutez
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   Il ouvre le périphérique en DSD natif au DSD64 et joue une seconde de
   silence DSD. Il doit réussir, et le convertisseur doit se verrouiller sur le
   DSD64.
5. Avec des fichiers DSD dans `test-music/`, `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   les joue à travers le moteur sur un périphérique simulé et compare les mots
   avec les octets du fichier (voir [Tests](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
