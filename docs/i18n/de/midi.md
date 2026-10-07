# MIDI-Controller

Fauste Player lässt sich mit MIDI-Controllern bedienen: Pad- und
Fader-Controllern, DJ-artigen Oberflächen oder Keyboards. Die Transport-Buttons
und der Lautstärke-Fader jedes Players können an eine Taste, ein Pad oder einen
Fader gebunden werden, und Tasten mit Lichtern zeigen, was jeder Player gerade
tut.

## Einschalten {#turning-it-on}

![Einstellungen, MIDI: der Schalter zum Einschalten von MIDI und die Liste der Aktionen mit je einem Lernen-Button](../../images/guide/settings-midi.png)

Öffne **Einstellungen → MIDI** und setze den Haken bei **MIDI-Controller
verwenden**. Die Liste unter **Eingangsports** zeigt jeden MIDI-Eingang des
Computers und ob er verbunden ist. Es werden nur die Controller geöffnet, die du
gebunden hast (manche Systeme geben einen Port jeweils nur einem Programm), dazu
jeder Eingang, während du ein Bedienelement lernst; das Programm hört nie seine
eigenen Ports ab. Controller können eingesteckt oder abgezogen werden, während
das Programm läuft: Alle paar Sekunden werden die Ports neu gesucht, und ein
Controller, der zurückkehrt, wird anhand seines Namens verbunden, mit neu
gesetzten Lichtern.

Unter Linux läuft MIDI über ALSA: Dein Benutzer muss den Sequencer
(`/dev/snd/seq`) öffnen dürfen, meist durch die Mitgliedschaft in der Gruppe
`audio`.

## Ein Bedienelement binden {#binding-a-control}

Für jeden Player gibt es eine Zeile pro Aktion: **Play / Weiter**, **Pause**,
**Stopp**, **Ausblenden**, **Neustart**, **Zurück**, **CUE** und **Lautstärke**.

1. Klicke in der Zeile auf **Lernen**.
2. Drücke die Taste oder bewege den Fader, den du möchtest (währenddessen steht
   dort **Ein Bedienelement bewegen…**). Eine Taste akzeptiert eine Taste, ein
   Pad oder einen Button, der eine Control-Change-Nachricht sendet;
   **Lautstärke** akzeptiert einen Fader oder Drehregler (Control Change) oder
   einen Pitch-Bend-Fader.
3. Die Zeile zeigt das Gerät und das Bedienelement, zum Beispiel
   `APC mini · Note 36, Kanal 1`.

War dieses Bedienelement bereits an eine andere Aktion gebunden, wandert es zu
dieser. `Esc` oder ein erneuter Klick auf den Button bricht das Lernen ab.
**Löschen** entfernt eine Bindung.

## Wie sich die Bedienelemente verhalten {#how-the-controls-behave}

- Eine Taste wirkt, wenn sie gedrückt wird (eine Taste oder ein Pad geht nach
  unten, oder ein Control Change geht durch die Mitte seines Bereichs nach
  oben), genau wie der Button des Players auf dem Bildschirm. Ein Button, der auf
  dem Bildschirm abgeblendet ist, bewirkt nichts.
- Ein Fader bewegt die Lautstärke auf derselben Skala wie der Fader auf dem
  Bildschirm. Um Sprünge zu vermeiden, übernimmt er erst, wenn er die aktuelle
  Lautstärke erreicht oder überschreitet (Soft Takeover); wird die Lautstärke
  auf dem Bildschirm geändert, muss der Fader sie erneut einholen. Nichts ändert
  sich, bis du ein Bedienelement bewegst: Der Start des Programms sendet nie
  etwas on Air.
- Ist **Tasten beleuchten (LED-Feedback)** eingeschaltet, leuchten gebundene
  Tasten: Play, solange der Player on Air ist, Pause blinkend, solange
  pausiert ist, CUE beim Vorhören sowie Stopp, Ausblenden, Neustart und Zurück,
  solange sie wirken können. Die Lichter gehen an den Ausgangsport des
  Controllers mit demselben Namen; ein anderer lässt sich in `config.json`
  festlegen (`midi.devices`).
