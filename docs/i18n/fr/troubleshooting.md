# Dépannage

## Pas de son {#no-sound}

1. Ouvrez **Paramètres → Sorties audio** et appuyez sur **Tester Main** pour
   le lecteur. Si vous entendez la tonalité, vérifiez le fader de volume du
   lecteur.
2. Si vous n'entendez rien, choisissez un autre périphérique ou une autre
   paire de canaux. Les changements de sorties prennent effet après un
   redémarrage : appuyez sur **Redémarrer maintenant** dans les Paramètres.
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
  [Sortie bit-perfect](bit-perfect.md#the-bp-badge) : volume à 100 %, aucun
  fondu, rien d'autre sur les mêmes sorties, un fichier sans perte qui a été
  analysé, et un périphérique tournant à la fréquence du fichier.
- **Un court silence avant une piste.** Le périphérique bit-perfect a été
  rouvert à la fréquence d'échantillonnage de la piste. Gardez la
  bibliothèque à une seule fréquence pour l'éviter.
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

## Coupures audio {#audio-dropouts}

La barre d'état avertit pendant 5 secondes après chaque coupure que
l'application détecte : **P1 : coupures audio (3)** quand le décodage d'un
lecteur n'a pas suivi le disque (le compte concerne la piste en cours de
lecture), et **&lt;périphérique&gt; : coupures du périphérique audio (2)**
quand le périphérique de sortie a manqué une échéance (un xrun). Le journal
enregistre aussi chacune, au plus une ligne toutes les 10 secondes par type,
avec le nombre survenu. Tous les systèmes audio ne signalent pas les xruns
(PulseAudio non ; le mode exclusif de Windows non).


- Augmentez la **taille du tampon** dans les Paramètres (et appuyez sur
  **Redémarrer maintenant**).
- Sous Linux, autorisez l'ordonnancement temps réel. L'application le demande
  au système via rtkit (D-Bus). L'appartenance au groupe `audio` avec une
  limite `rtprio` fonctionne aussi.
- Évitez les disques réseau pour la musique diffusée à l'antenne.

## « L'interface a rencontré une erreur » {#the-interface-hit-an-error}

Une erreur d'affichage a été interceptée. L'audio n'est pas affecté. Appuyez
sur **Redémarrer l'interface**. Merci de la signaler avec les journaux.

## Journaux et rapports de plantage {#logs-and-crash-reports}

Voir [Données et sauvegardes](data-and-backups.md) pour le dossier des
journaux. Il y a un fichier journal par jour, et les 14 derniers sont
conservés. Les rapports de plantage sont enregistrés sous le nom
`crash-<time>.txt`. Définissez `RUST_LOG=debug` dans l'environnement pour
plus de détails. Joignez les deux fichiers quand vous signalez un bogue.
