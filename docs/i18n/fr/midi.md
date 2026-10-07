# Surfaces de contrôle MIDI

Fauste Player peut se piloter depuis des contrôleurs MIDI : contrôleurs à
pads et à faders, surfaces de type DJ ou claviers. Les boutons de transport
et le fader de volume de chaque lecteur peuvent être associés à un bouton, à
une touche ou à un fader, et les boutons lumineux montrent ce que fait chaque
lecteur.

## L'activer {#turning-it-on}

![Paramètres, MIDI : l'interrupteur pour activer le MIDI, et la liste des actions avec un bouton Apprendre chacune](../../images/guide/settings-midi.png)

Ouvrez **Paramètres → MIDI** et cochez **Utiliser des surfaces de contrôle
MIDI**. La liste sous **Ports d'entrée** montre chaque entrée MIDI de
l'ordinateur et si elle est connectée. Seuls les contrôleurs que vous avez
associés sont ouverts (certains systèmes n'attribuent un port qu'à un
programme à la fois), plus toutes les entrées pendant que vous apprenez une
commande ; le programme n'écoute jamais ses propres ports. Les contrôleurs
peuvent être branchés ou débranchés pendant que le programme tourne : toutes
les quelques secondes, les ports sont recherchés de nouveau, et un contrôleur
qui revient est reconnecté par son nom, avec ses voyants rétablis.

Sous Linux, le MIDI passe par ALSA : votre utilisateur doit être autorisé à
ouvrir le séquenceur (`/dev/snd/seq`, généralement en faisant partie du
groupe `audio`).

## Associer une commande {#binding-a-control}

Pour chaque lecteur, il y a une ligne par action : **Lecture / Suivante**,
**Pause**, **Stop**, **Stop en fondu**, **Reprendre au début**,
**Précédente**, **CUE** et **Volume**.

1. Cliquez sur **Apprendre** sur la ligne.
2. Appuyez sur le bouton ou bougez le fader voulu (il est indiqué **Bougez
   une commande…** entre-temps). Un bouton accepte une touche, un pad ou un
   bouton qui envoie un changement de commande (control change) ;
   **Volume** accepte un fader ou un potentiomètre (un control change) ou un
   fader à pitch bend.
3. La ligne affiche le périphérique et la commande, par exemple
   `APC mini · Note 36, canal 1`.

Si cette commande était déjà associée à une autre action, elle passe à
celle-ci. `Esc` ou un nouveau clic sur le bouton annule l'apprentissage.
**Effacer** supprime une association.

## Comportement des commandes {#how-the-controls-behave}

- Un bouton agit quand on appuie dessus (une touche ou un pad qui s'enfonce,
  ou un control change qui franchit le milieu de sa plage), exactement comme
  le bouton du lecteur à l'écran. Un bouton grisé à l'écran ne fait rien.
- Un fader règle le volume sur la même échelle que le fader à l'écran. Pour
  éviter les sauts, il ne prend la main que lorsqu'il atteint ou dépasse le
  volume actuel (reprise douce, ou soft takeover) ; si le volume est modifié
  à l'écran, le fader doit le rattraper de nouveau. Rien ne change tant que
  vous ne bougez pas une commande : démarrer le programme n'envoie jamais
  rien à l'antenne.
- Avec **Allumer les boutons (retour LED)** activé, les boutons associés
  s'allument : Lecture quand le lecteur est à l'antenne, Pause clignotant
  pendant la pause, CUE pendant la pré-écoute, et Stop, Stop en fondu,
  Reprendre au début et Précédente tant qu'ils peuvent agir. Les voyants sont
  envoyés vers le port de sortie du contrôleur portant le même nom ; un autre
  peut être défini dans `config.json` (`midi.devices`).
