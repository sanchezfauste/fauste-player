# Dépannage

## Pas de son {#no-sound}

1. Ouvrez **Paramètres → Sorties audio** et appuyez sur **Tester Main** pour
   le lecteur. Si vous entendez la tonalité, vérifiez le fader de volume du
   lecteur.
2. Si vous n'entendez rien, choisissez un autre périphérique ou une autre
   paire de canaux. Un changement des sorties s'applique dès que plus rien ne
   joue dessus ; **Appliquer maintenant** dans les Paramètres l'applique
   aussitôt.
3. Sous Linux, préférez **PipeWire** ou **PulseAudio** dans Paramètres →
   Sorties audio → Système audio. Ils partagent la carte son avec d'autres
   programmes. **ALSA** parle directement à la carte et peut la trouver
   occupée.
4. **JACK** apparaît comme indisponible (« no output device ») quand aucun
   serveur JACK ne tourne. Démarrez le serveur (par exemple avec QjackCtl) et
   redémarrez l'application. Réglez le serveur JACK sur la fréquence
   d'échantillonnage des Paramètres (48 kHz par défaut) : JACK tourne à une
   seule fréquence pour tous les programmes.
5. **PipeWire** n'est pas proposé par les archives téléchargeables ; elles
   atteignent PipeWire via son service PulseAudio, qui fonctionne de la même
   façon. Il est disponible dans les versions compilées avec la fonctionnalité
   `pipewire`.

## Bit-perfect {#bit-perfect}

- **Le badge BP reste éteint.** Vérifiez chaque condition dans
  [Sortie bit-perfect](bit-perfect.md#the-bp-badge) : aucun fondu, rien
  d'autre sur les mêmes sorties, un fichier sans perte qui a été analysé, et
  un périphérique tournant à la fréquence du fichier.
- **Un court silence avant une piste.** Le périphérique bit-perfect a été
  rouvert à la fréquence d'échantillonnage de la piste. Cela n'arrive que
  lorsque la fréquence change : les pistes à la même fréquence s'enchaînent
  sans interruption. Gardez la bibliothèque à une seule fréquence pour
  l'éviter. Voir [Ce que cela coûte](bit-perfect.md#what-it-costs).
- **Les pistes ne se mixent pas, et PLAY ne fait pas de fondu.** Sur un
  lecteur dont la sortie Main est bit-perfect, chaque piste est lue jusqu'à sa
  fin puis la suivante démarre, et PLAY / SUIV. et Précédente basculent
  aussitôt : il n'y a ni fondus enchaînés ni segues sur un tel lecteur.
  Désactivez le bit-perfect pour le périphérique, ou déplacez le lecteur sur
  une autre sortie, pour mixer de nouveau. Voir
  [Ce que cela coûte](bit-perfect.md#what-it-costs).
- **Le fader de volume ne bouge pas.** La sortie Main du lecteur est
  bit-perfect, ce qui maintient le volume à 100 % ; l'infobulle du fader le
  dit. Les faders MIDI et l'API à distance ne peuvent pas le bouger non plus.
  Voir [Ce que cela coûte](bit-perfect.md#what-it-costs).
- **Une piste est lue rééchantillonnée, et le journal indique que le
  périphérique est occupé (Linux).** Pour changer de fréquence, l'application
  ferme le périphérique puis le rouvre. À ce moment, le serveur de son
  (PipeWire) peut prendre la carte. L'application réessaie plusieurs fois ;
  si la carte est toujours occupée, la piste est lue à la fréquence actuelle
  du périphérique, et la piste suivante redemande sa fréquence. Pour donner la
  carte à l'application seule, ouvrez les réglages sonores du système et
  mettez le profil de cette carte sur **Off** (ou **Pro Audio**), afin que le
  serveur de son laisse son périphérique `hw:` tranquille. Le nombre
  d'essais et l'attente entre eux sont `tuning.device_busy_retries` et
  `tuning.device_busy_retry_ms` dans le fichier de configuration.
- **Le périphérique joue, mais le badge BP reste éteint (Windows ou
  macOS).** L'accès exclusif a été refusé, et le périphérique joue en mode
  partagé.
  - Windows : un autre programme détient peut-être le périphérique en
    exclusif, ou le contrôle exclusif est désactivé dans les propriétés
    avancées du périphérique.
  - macOS : un autre programme détient peut-être le périphérique en mode hog,
    ou le périphérique n'offre ses fréquences que sous forme de plage continue
    (la plupart des interfaces listent des fréquences fixes).
- **Un périphérique `hw:` ne peut pas être ouvert (Linux).**
  - Un serveur de son détient peut-être la carte. Arrêtez-le, ou configurez le
    serveur pour qu'il laisse cette carte tranquille, et redémarrez
    l'application.
  - Certains DAC USB n'acceptent que des échantillons 24 bits compactés
    (`S24_3LE`), que la bibliothèque audio ne prend pas en charge. Utilisez
    plutôt cette carte via `plughw:` (pas bit-perfect).

### DSD {#dsd}

- **Une piste DSD est lue convertie alors que le périphérique est réglé sur
  DoP ou DSD natif.** Le journal en donne la raison (« DSD converted to PCM »
  et la cause) pour ces causes : le volume du lecteur n'est pas à 100 %, autre
  chose joue sur le périphérique, plus de deux canaux, ou un périphérique qui
  refuse la fréquence (le DoP exige la fréquence DSD divisée par 16, par
  exemple 176,4 kHz pour le DSD64) ou n'a aucun format 24 ou 32 bits. Une
  piste qui n'a pas encore été analysée est convertie silencieusement, sans
  ligne de journal : analysez-la (Paramètres → Analyse) et relancez-la.
- **Seule la première piste d'un album DSD sort en DSD.** C'est le réglage de
  mixage par défaut : les pistes que le lecteur démarre lui-même sont lues
  converties. Choisissez **Garder le DSD et couper les autres sources** dans
  Paramètres → Sorties audio pour les garder en DSD. Voir
  [DSD](bit-perfect.md#dsd).
- **L'en-tête affiche DSD mais le convertisseur joue du bruit ou ne se
  verrouille pas.** Le convertisseur ne reconnaît pas le DoP (ou le format
  natif). Remettez le périphérique sur **Convertir en PCM**.
- **Un clic quand une piste DSD démarre, s'arrête ou quitte le DSD.** Le
  convertisseur a besoin de plus de silence DSD : augmentez **Silence DSD**
  (200 ms par défaut) dans Paramètres → Sorties audio, Avancé.
- **Les autres lecteurs ou cartouches sont silencieux sur le périphérique.**
  Une piste DSD est en lecture avec **Garder le DSD et couper les autres
  sources** ; le badge **Autres coupées** s'affiche. Ils sonnent de nouveau à
  la fin de la piste.

## Alerte « Sortie perdue » {#output-lost-alert}

La barre d'état affiche **Sortie perdue : &lt;périphérique&gt;** quand un
périphérique cesse de répondre. Les lecteurs continuent de compter et de
mixer sur une horloge interne, si bien que l'automatisation ne se bloque pas.
Le périphérique est réessayé toutes les 2 secondes et reprend la main à son
retour. Rebranchez le câble ou rallumez l'interface.

### « Sortie perdue » qui ne disparaît jamais, avec une sortie `hw:` directe {#output-lost-that-never-clears-with-a-direct-hw-output}

Une carte son utilisée via une sortie ALSA `hw:` directe (par exemple une
sortie bit-perfect) est détenue par Fauste Player seul : le serveur de son
(PipeWire ou PulseAudio) ne peut pas l'utiliser en même temps. Si une autre
sortie passe par le périphérique par défaut du serveur de son et que ce
périphérique par défaut est la même carte, cette sortie ne démarre jamais et
reste en **Sortie perdue**. Le journal indique une fois « output device opened
but never started ».

Utilisez un seul chemin par carte : faites passer toutes les sorties de cette
carte par le même périphérique `hw:` (avec des canaux différents si
nécessaire), ou choisissez une autre carte comme sortie par défaut du
serveur de son dans les réglages sonores de votre système.

## Une piste affiche une icône d'avertissement ou un fichier avec une croix {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

Le fichier est introuvable (déplacé, supprimé, démonté : un fichier avec une
croix) ou ne peut pas être décodé (un signe d'avertissement). Les lecteurs
l'ignorent. Survolez la ligne pour voir lequel des deux, ainsi que le chemin
du fichier.

Un fichier introuvable est recherché de nouveau toutes les 30 secondes
(`tuning.missing_recheck_ms` dans le fichier de configuration) : quand le
disque est monté ou que le fichier est remis en place, la piste redevient
lisible d'elle-même. Un fichier qui ne peut pas être décodé est revérifié de
lui-même sur la même minuterie, d'après sa taille et sa date de modification :
il n'est pas décodé de nouveau à moins que l'une des deux ait changé, par
exemple quand une copie se termine. Pour le vérifier tout de suite, utilisez
**Réanalyser** dans le menu de sa ligne, ou **Paramètres → Analyse →
Réanalyser toutes les pistes** pour toute la bibliothèque.

Un WAV ou un FLAC qui apparaît comme illisible, ou qui jouerait comme un
bruit fort, peut être un rip de CD DTS : le fichier contient un flux DTS au
lieu d'audio. L'application le refuse (le journal dit « encoded bitstream
stored as PCM ») et ne décode pas le DTS. Convertissez-le d'abord en PCM
ordinaire avec un autre outil.

## Un dossier surveillé affiche un avertissement {#a-watched-folder-shows-a-warning}

Une catégorie qui surveille un dossier affiche ⟳, d'une autre couleur quand le
dossier n'est pas suivi ; survolez-le pour la raison. Voir [Dossiers
surveillés](library.md#watched-folders).

- **Introuvable ou illisible** : le disque ou le partage n'est pas monté ou
  n'est pas joignable. Un dossier vide qui devrait contenir des pistes de la
  bibliothèque compte aussi comme introuvable, car un partage non monté
  ressemble souvent à cela. Rien n'est marqué ni retiré ; le dossier est
  recherché de nouveau toutes les 30 secondes et analysé quand il apparaît. Si
  vous avez vidé le dossier exprès, supprimez ses pistes en les sélectionnant
  dans le tableau de la bibliothèque et en choisissant **Retirer de la
  bibliothèque…**, ou en choisissant **Réanalyser** sur elles, ce qui marque
  comme manquantes celles dont le fichier a disparu, afin que **Retirer les
  pistes manquantes…** puisse les retirer.
- **Les changements ne sont pas suivis** : le système n'a pas voulu le
  surveiller, en général un dossier avec un très grand nombre de
  sous-dossiers. Le dossier est quand même analysé à chaque démarrage et à
  chaque nouvelle tentative, si bien que les nouveaux fichiers apparaissent
  alors, et non aussitôt. Sous Linux, une arborescence plus grande que la
  limite de surveillances par utilisateur demande un
  `fs.inotify.max_user_watches` plus élevé (par exemple `sudo sysctl
  fs.inotify.max_user_watches=524288` ; rendez-le permanent dans
  `/etc/sysctl.d/`).
- **Pas un chemin complet** : le dossier enregistré a été modifié à la main en
  un chemin qui n'est pas complet. Choisissez **Changer le dossier
  surveillé…** pour le choisir de nouveau.
- **Un partage réseau n'affiche aucun avertissement, mais les fichiers
  ajoutés depuis un autre ordinateur n'apparaissent pas** : sur un partage
  réseau (SMB/CIFS, NFS), le système accepte en général la surveillance mais
  ne signale que les changements faits depuis cet ordinateur. Les changements
  faits ailleurs sont vus au prochain démarrage, pas pendant que l'application
  tourne.
- **Un dossier avec des milliers de fichiers** met un moment à être analysé au
  démarrage ; la progression de l'analyse le montre.
- Les pistes dont le fichier a quitté un dossier surveillé restent dans la
  bibliothèque, marquées comme manquantes. Retirez-les avec **Retirer les
  pistes manquantes…** dans le menu de la catégorie.

## Coupures audio {#audio-dropouts}

La barre d'état avertit pendant 5 secondes après chaque coupure que
l'application détecte : **P1 : coupures audio (3)** quand le décodage d'un
lecteur n'a pas suivi le disque (le compte concerne la piste en cours de
lecture), et **&lt;périphérique&gt; : coupures du périphérique audio (2)**
quand le périphérique de sortie a manqué une échéance (un xrun). Le journal
enregistre aussi chacune, au plus une ligne toutes les 10 secondes par type,
avec le nombre survenu. Tous les systèmes audio ne signalent pas les xruns
(PulseAudio non ; le mode exclusif de Windows non).


- Augmentez la **taille du tampon** dans les Paramètres (elle s'applique dès
  que plus rien ne joue sur ce périphérique, ou aussitôt avec **Appliquer
  maintenant**).
- Via un serveur de son (PulseAudio, le périphérique ALSA `default`), chaque
  sortie demande déjà au moins 50 ms quelle que soit la taille du tampon, afin
  qu'un ordinateur occupé ne s'entende pas ; les Paramètres l'indiquent à côté
  de la latence. Sur un ordinateur ancien ou très chargé, augmentez
  `shared_buffer_ms` dans la section `outputs` du fichier de configuration
  (par exemple à 100). Le système garde environ deux de ces blocs en file, si
  bien que les boutons des cartouches et de lecture réagissent alors environ
  deux fois plus tard que l'augmentation. Pour la sortie d'antenne, l'entrée
  matérielle propre au périphérique (`hw:` sous Linux) évite tout à fait le
  serveur de son.
- Sous Linux, autorisez l'ordonnancement temps réel. L'application le demande
  au système via rtkit (D-Bus), et le journal enregistre un refus. Beaucoup de
  bureaux le refusent aux applications.
- Évitez les disques réseau pour la musique diffusée à l'antenne.

## L'interface est lente sur un ordinateur ancien {#the-interface-is-slow-on-an-old-computer}

Pendant qu'un son joue, l'interface dessine au plus 30 images par seconde
(`max_fps` dans la section `ui` du fichier de configuration), et les clics et
les touches reçoivent une réponse immédiate. Sur un ordinateur ancien ou
occupé, abaissez-le (par exemple à 20) pour laisser plus de processeur à
l'audio ; 0 dessine chaque image que l'écran montre. Les vumètres mesurent la
même chose quelle que soit la cadence ; ils sont seulement dessinés moins
souvent.

## Les fichiers ne peuvent pas être déposés sous Linux {#files-cannot-be-dropped-on-linux}

La fenêtre n'accepte les fichiers glissés depuis un gestionnaire de fichiers
que lorsqu'elle s'exécute sous X11. Par défaut, c'est le cas, y compris sur un
bureau Wayland (via XWayland, que tous les bureaux ont). Si
`ui.window_system` dans `config.json` vaut `"Wayland"`, la fenêtre est une
fenêtre Wayland native, qui peut paraître plus nette avec une mise à l'échelle
fractionnaire de l'affichage, mais on ne peut pas y déposer de fichiers :
utilisez plutôt **+ Ajouter**. Remettez-le sur `"Auto"` (la valeur par défaut)
ou `"X11"` et redémarrez Fauste Player.

## Un contrôleur MIDI ne répond pas {#a-midi-controller-does-not-respond}

Si **Paramètres → MIDI** montre le contrôleur comme connecté mais que ses
boutons et ses faders ne font rien, appuyez sur **Reconnecter les ports** à cet
endroit : le programme ferme les ports MIDI et les rouvre, comme si vous
débranchiez puis rebranchiez le contrôleur. Si cela n'aide pas, débranchez le
contrôleur et rebranchez-le.

Dans les deux cas, le journal dit ce qui s'est passé. Il liste les ports MIDI
chaque fois qu'ils changent, chaque port ouvert et fermé (avec la raison), un
port qui ne peut pas être ouvert (une fois, jusqu'à ce qu'il s'ouvre ou
disparaisse), le premier message reçu sur chaque connexion, le premier message
ignoré sur chaque connexion avec la raison (aucune association, un fader qui
n'a pas encore atteint le volume, une action qui ne peut pas agir pour
l'instant) et le premier message de voyant qui a échoué. Un contrôleur
connecté qui n'enregistre jamais de premier message n'envoie rien que le
programme reçoive. Pour chaque message ignoré et chaque nouvelle tentative
échouée, démarrez le programme avec `RUST_LOG=info,fp_control=debug`.

## « L'interface a rencontré une erreur » {#the-interface-hit-an-error}

Une erreur d'affichage a été interceptée. L'audio n'est pas affecté. Appuyez
sur **Redémarrer l'interface**. Merci de la signaler avec les journaux.

## Journaux et rapports de plantage {#logs-and-crash-reports}

Voir [Données et sauvegardes](data-and-backups.md) pour le dossier des
journaux. Il y a un fichier journal par jour. Les lignes identiques qui se
répètent l'une après l'autre sont écrites une seule fois, suivies d'une ligne
telle que `previous message repeated 12 times` (pendant une longue série de
répétitions, cette ligne est écrite chaque minute). Les lignes qui diffèrent,
par exemple parce qu'elles portent un compteur, ne sont pas regroupées. Les
messages de connexion courants du client PulseAudio sont omis par défaut.
Définissez `RUST_LOG=debug` dans l'environnement pour plus de détails (il
remplace le filtre par défaut, `info,pulseaudio=warn`).

Le journal est borné : le fichier d'un jour cesse de grossir à 10 Mio (sa
dernière ligne le dit ; le reste de ce jour n'est pas journalisé, ce
qu'entraîne un déluge de lignes différentes) et les 14 derniers fichiers sont
conservés. Modifiez-les dans le fichier de configuration avec
`limits.max_log_file_mb` (de 1 à 1024), `limits.max_log_files` (de 2 à 365) et
`limits.log_repeat_summary_secs` (de 1 à 3600, 60 par défaut, la fréquence à
laquelle une longue série de répétitions est signalée) ; ils s'appliquent au
prochain démarrage. Les rapports de plantage sont enregistrés sous le nom
`crash-<time>.txt` dans le même dossier et sont limités séparément
(`limits.max_crash_reports` par exécution). Joignez les deux fichiers quand
vous signalez un bogue.
