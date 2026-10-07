# Marqueurs et mixage

Chaque piste a jusqu'à cinq **marqueurs**, en secondes :

| Marqueur | Signification | Comment il est défini |
|---|---|---|
| Cue-in | Où la lecture commence | Automatique : juste avant le premier son au-dessus du seuil de rognage |
| Cue-out | Où la piste se termine | Automatique : juste après le dernier son au-dessus du seuil de rognage |
| MIX (début de l'enchaînement) | Où la piste suivante démarre en mode continu | Automatique (voir plus bas) |
| Début de l'outro | Où commence la fin de la piste | Automatique (voir plus bas) |
| Fin de l'intro | Fin de l'introduction parlée par-dessus | À la main, ou depuis un tag `INTRO` dans le fichier |

Les marqueurs placés à la main l'emportent toujours : une nouvelle analyse ne
les remplace jamais.

## Modifier les marqueurs {#editing-markers}

Sur la forme d'onde d'un lecteur, ou sur celle de sa fenêtre CUE (le même
menu et les mêmes poignées ; un changement apparaît dans les deux en même
temps) :

- **Faites un clic droit** à l'endroit où vous voulez un marqueur et
  choisissez **Placer le cue-in ici**, **Placer la fin de l'intro ici**,
  **Placer le début de l'outro ici**, **Placer le point MIX ici** ou **Placer
  le cue-out ici**. **Rétablir les marqueurs automatiques** supprime les
  marqueurs que vous avez placés, et la piste est analysée de nouveau.
- **Maintenez Alt** (Option sous macOS) : des poignées apparaissent sur les
  marqueurs. Faites-en glisser une pour la déplacer ; le temps s'affiche
  pendant le glissement. Un glissement ne déplace jamais la tête de lecture.

Le cue-in doit rester avant le cue-out. Les autres marqueurs sont conservés
entre eux. Les changements sur la piste à l'antenne s'appliquent aussitôt à
sa prochaine transition.

## Le tag INTRO {#the-intro-tag}

Un fichier peut porter la durée de son intro dans un tag `INTRO`, en secondes
(`12.5`) ou en `m:ss`. Le nom peut être écrit en minuscules ou majuscules
(`INTRO`, `Intro`). Ce peut être une trame de texte utilisateur ID3v2 (MP3,
WAV, AIFF, DSF), un commentaire Vorbis, Opus ou FLAC, un élément APE (WavPack,
Monkey's Audio) ou un atome libre MP4. Il est lu pendant l'analyse. Une fin
d'intro placée à la main l'emporte toujours.

## Comment les marqueurs automatiques sont trouvés {#how-the-automatic-markers-are-found}

L'analyse mesure les crêtes de la piste par pas de 10 ms et son loudness dans
de courtes fenêtres (50 ms par défaut).

- **Cue-in / cue-out :** seul le quasi-silence du début et de la fin est
  ignoré : tout ce dont la crête atteint le *seuil de rognage* (−60 dBFS par
  défaut), sur l'un ou l'autre canal, est conservé, avec une *marge de
  rognage* (20 ms par défaut) autour. Les fondus d'entrée doux, les queues
  discrètes et les sons courts ne sont jamais coupés.
- **MIX :** l'analyse trouve le dernier point où la piste est encore à moins
  de la *baisse pour l'enchaînement* (15 dB par défaut) sous son propre
  loudness typique, si bien que des masters forts et faibles avec le même
  fondu se mixent de la même façon. Ce point n'est jamais à plus de la *durée
  maximale de l'enchaînement* (4 s par défaut) avant le cue-out, afin que les
  recouvrements restent courts.
- **Outro :** l'analyse remonte depuis le cue-out et trouve où le niveau
  descend de plus de la *baisse de niveau de l'outro* (6 dB par défaut) sous
  le loudness médian de la piste. L'outro n'est jamais plus longue que 30 s
  par défaut.
- Les pistes plus courtes que la *durée minimale pour les marqueurs de mix et
  d'outro* (60 s par défaut), comme les jingles et les publicités, n'ont ni
  MIX ni outro.

### Longs enregistrements {#long-recordings}

Une émission entière (une, quatre heures ou plus) est analysée comme une
chanson, pendant sa lecture si besoin : un fichier FLAC ou Opus de 4 heures
prend moins d'une minute sur un ordinateur actuel, et la mémoire ne croît pas
avec la durée. Se déplacer à n'importe quel point, même près de la fin, est
immédiat. La forme d'onde et les marqueurs sont conservés dans le cache
d'analyse jusqu'à environ 16 heures d'audio ; un fichier plus long fonctionne
aussi, mais il est analysé de nouveau à chaque démarrage de Fauste Player.

Toutes ces valeurs se trouvent dans **Paramètres → Analyse**. Après les avoir
modifiées, les pistes sont analysées de nouveau automatiquement.

## Ce que le lecteur en fait {#what-the-player-does-with-them}

- **Mode continu avec mixage automatique activé :** au point MIX, la piste
  suivante démarre à plein niveau pendant que la piste en cours s'estompe
  jusqu'à son cue-out. Le recouvrement est précis à l'échantillon près.
- **Mode continu sans point MIX,** ou avec le mixage automatique désactivé :
  la piste suivante démarre exactement au cue-out, sans interruption.
- **Mode single**, ou **Stop à la fin** : le lecteur s'arrête au cue-out.
- **Appuyer sur PLAY pendant l'antenne :** la piste suivante démarre aussitôt
  et la piste en cours s'estompe pendant la *durée du fondu* (1 s par
  défaut).
- **Utiliser le cue-in et le cue-out désactivé** (Paramètres → Lecteurs) :
  chaque lecteur joue chaque piste de 0 à la fin du fichier. Le cue-in et le
  cue-out, automatiques et manuels, sont conservés, et la forme d'onde les
  dessine comme des lignes atténuées. Le point MIX, l'intro et l'outro
  fonctionnent toujours, au sein du fichier entier ; l'**enchaînement
  automatique** est un interrupteur distinct. Les comptes à rebours, la
  colonne de durée, les totaux de playlist dans les Paramètres et les temps de
  l'API à distance suivent la même plage. Les cartouches utilisent toujours
  leurs propres cue-in et cue-out. Changer ce réglage ne redémarre, ne
  déplace ni n'arrête jamais une piste en cours de lecture ; la piste
  suivante est préparée de nouveau.

Une piste peut être lue avant la fin de son analyse. Jusque-là, elle joue du
début à la fin du fichier, sans point MIX.
