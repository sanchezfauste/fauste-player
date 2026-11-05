# Layout

Ordne das Hauptfenster für deine Schicht an: Gib den Playern und der Cartwall
die gewünschten Größen, setze die Cartwall unter die Player oder neben sie,
blende die Panels aus, die du nicht brauchst, und maximiere eines für einen
Moment. Das Layout wird mit der Sitzung gespeichert und kommt beim nächsten
Start wieder; eine neue Installation beginnt mit allen Playern gleich breit
und der Cartwall unter ihnen.

## Panels ein- und ausblenden {#show-and-hide-panels}

Die Kacheln links in der oberen Leiste, **P1** … **Pn** und **Cartwall**,
zeigen, welche Panels auf dem Bildschirm sind: Eine leuchtende Kachel ist
sichtbar. Klicke auf eine Kachel, um ihr Panel aus- oder einzublenden. Bei
vielen Playern scrollen die Kacheln seitwärts (drehe das Mausrad über ihnen),
und der Layout-Button bleibt im Blick. Die anderen Panels nehmen den frei
werdenden Platz ein, und ein wieder eingeblendeter Player kommt in seiner
Breite zurück.

Das Ausblenden nimmt ein Panel nur vom Bildschirm. Ein ausgeblendeter Player
spielt weiter, und seine Zifferntaste, MIDI, die Remote-API, sein CUE-Fenster
und die Automatisierung steuern ihn weiterhin. Eine Kachel zeigt den Zustand
ihres Panels, ob es eingeblendet oder ausgeblendet ist: einen roten Punkt,
solange ein Player spielt (oder die Cartwall, solange ein Cart spielt), und
zwei bernsteinfarbene Balken, wie bei einem Pause-Button, solange ein Player
pausiert ist.

Das Ausblenden eines Players hebt seinen ausgewählten Track auf, sodass
**Entf** keinen Track entfernen kann, den du nicht siehst. Dateien, die abseits
einer Liste auf das Fenster gezogen werden, gehen an den zuletzt verwendeten
Player oder an den ersten sichtbaren; sind alle Player ausgeblendet, werden
sie abgelehnt, und die Statusleiste sagt es.

Die Kachel **Bibliothek** danach öffnet und schließt das
[Bibliotheksfenster](library.md), das über den Panels schwebt; sie ist kein
Panel, sodass die Regel der zuletzt leuchtenden Kachel unten für sie nicht
gilt. Auf einem Telefon wird sie nicht angezeigt.

Mindestens ein Panel bleibt sichtbar: Die letzte leuchtende Kachel lässt sich
nicht anklicken. Solange die Cartwall ausgeblendet ist, bewirkt das Kürzel, das
sie ein- oder ausklappt, nichts; klicke auf **Cartwall**, um sie einzublenden.

## Die Player in der Größe ändern {#resize-the-players}

Ziehe die Lücke zwischen zwei Playern, um Breite von einem auf den anderen zu
verschieben. Die übrigen Player bewegen sich nicht, und kein Player wird
schmaler, als seine Bedienelemente brauchen. Doppelklicke auf die Lücke, um
allen Playern wieder dieselbe Breite zu geben.

Ist das Fenster zu schmal für alle sichtbaren Player in ihrer kleinsten
Größe, scrollt die Reihe der Player seitwärts, wie sie es immer getan hat.

## Die Cartwall verschieben und in der Größe ändern {#move-and-resize-the-cartwall}

Die Cartwall sitzt unter den Playern oder links oder rechts im Fenster in
voller Höhe: Wähle **Cartwall unten**, **Cartwall links** oder **Cartwall
rechts** im [Layout-Menü](#the-layout-menu). Unter den Playern ist sie so hoch,
wie ihre Zeilen es brauchen, höchstens sechs Zehntel der Höhe zwischen der
oberen Leiste und der Statusleiste; neben ihnen nimmt sie zunächst drei Zehntel
der Breite ein.

Die Cartwall ist nie schmaler als ihr Kopf (**Alle stoppen**, ihr Titel und ein
Seiten-Tab) oder als ihre Spalten aus Buttons in kleinster Größe. Ist das
Fenster zu schmal für sie und alle sichtbaren Player in ihrer kleinsten Größe,
scrollt die Reihe der Player seitwärts.

Ziehe die Lücke zwischen der Cartwall und den Playern, um sie größer oder
kleiner zu machen: nach oben oder unten, wenn sie unter den Playern liegt,
seitwärts, wenn sie daneben liegt. Die Player behalten den Platz, den ihre
Bedienelemente brauchen. Doppelklicke auf die Lücke, um zur Startgröße
zurückzukehren: so hoch, wie ihre Zeilen es brauchen, unter den Playern, drei
Zehntel der Breite daneben. Die Größe wird mit dem Layout gespeichert; wird die
Cartwall verschoben, beginnt sie wieder mit ihrer Startgröße.

Sobald du die Cartwall in der Größe änderst oder solange sie neben den Playern
sitzt, wachsen ihre Buttons, um sie zu füllen; Namen und Zeiten behalten ihre
Größe, in der Mitte jedes Buttons. Ein Name wird zwischen seinen Wörtern
umbrochen; ein Wort, das für seinen Button zu lang ist, endet mit einer
Ellipse (…), statt über Zeilen hinweg getrennt zu werden. Ist die Cartwall
kleiner, als ihre Zeilen und Spalten bei den kleinsten Buttons brauchen,
scrollt sie.

Klicke auf **CARTWALL**, um die Cartwall einzuklappen. Unter den Playern bleibt
nur ihr Kopf; neben ihnen wird sie zu einem schmalen Streifen in Fensterhöhe,
mit einem Pfeil zum erneuten Ausklappen und, solange ein Cart spielt, einem
roten Punkt. Ausgeklappt hat sie ihre Größe zurück. Solange sie eingeklappt
ist, bewirkt die Lücke daneben nichts.

## Ein Panel maximieren {#maximize-a-panel}

Der Kopf jedes Players und der der Cartwall endet mit zwei kleinen Buttons:
**Ausblenden** (ein Minus) und **Maximieren** (nach außen zeigende Ecken;
**Wiederherstellen**, nach innen zeigende Ecken, am maximierten Panel). Fahre
mit der Maus darüber für den Namen. Sie sind abgeblendet, solange das Layout
gesperrt ist, **Ausblenden** am letzten sichtbaren Panel und **Maximieren**,
wenn nur ein Panel sichtbar ist und bei einer eingeklappten Cartwall. Dieselben
zwei Aktionen stehen im Menü eines Rechtsklicks auf den Hintergrund des Kopfes
(nicht auf einen seiner Buttons):

- **Maximieren** gibt diesem Panel den ganzen Platz: Die anderen Panels
  schrumpfen auf ihre kleinste Größe. Das gilt nur für den Moment und wird
  nicht gespeichert. **Wiederherstellen** im selben Menü beendet es, ebenso das
  Aus- oder Einblenden eines Panels, das Sperren des Layouts, das Verschieben
  der Cartwall, das Ändern der Anzahl der Player und das Ziehen oder
  Doppelklicken einer Lücke zwischen Panels; die Größen von vorher kommen
  zurück. Eine eingeklappte Cartwall lässt sich nicht maximieren, und das
  Einklappen einer maximierten Cartwall beendet es.
- **Ausblenden** blendet das Panel aus, wie seine Kachel in der oberen Leiste.

## Das Layout-Menü {#the-layout-menu}

Der Layout-Button rechts von den Kacheln öffnet ein Menü mit:

- **Cartwall unten**, **Cartwall links**, **Cartwall rechts**: wo die Cartwall
  sitzt;
- **Layout sperren**: Solange es angehakt ist, ändert nichts das Layout aus
  Versehen: die Kacheln, die übrigen Einträge dieses Menüs, die Lücken zwischen
  den Panels und **Maximieren** und **Ausblenden** der Köpfe bewirken nichts.
  Die Wiedergabe sowie das Ein- oder Ausklappen der Cartwall funktionieren wie
  gewohnt. Die Sperre wird mit dem Layout gespeichert;
- **Player gleich breit**: alle Player gleich breit;
- **Layout zurücksetzen**: zurück zum Layout einer neuen Installation.
