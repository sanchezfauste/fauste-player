# Playlists

## Tabs {#tabs}

Jeder Player hat eine Reihe von Tabs, einen pro Playlist. Alle Player sehen
dieselben Playlists; jeder Player wählt, welche er anzeigt. Ein Punkt auf einem
Tab zeigt, wo die Tracks des Players liegen: **rot** für den Track, der on Air
ist, **grün** für den nächsten.

Die Tabs teilen sich die Breite des Players. Ein Name, der nicht passt, endet
auf „…“; fahre mit der Maus über den Tab, um ihn vollständig zu lesen. Bei
vielen Playlists schrumpfen die Tabs nur bis zu einer Mindestbreite, an den
Enden der Reihe erscheinen Pfeile, und das Mausrad über den Tabs scrollt sie.
Der Tab, den du wählst, und der, den ein Player anzeigt, wird in den sichtbaren
Bereich gebracht.

Das Wechseln der Tabs ändert nie, was on Air ist oder was als Nächstes kommt.
Endet ein Track, macht der Player in der Playlist weiter, die diesen Track
enthält.

Playlists werden in den [Einstellungen](settings.md) erstellt, umbenannt und
gelöscht. Die letzte Playlist und eine Playlist mit einem Track on Air lassen
sich nicht löschen.

## Die Track-Tabelle {#the-track-table}

![Eine Playlist: gespielte Tracks abgeblendet, der Track on Air rot, der nächste Track grün, und die Fußzeile mit der Restzeit](../../images/guide/playlist.png)

| Spalte | Inhalt |
|---|---|
| `#` | Position, mit führenden Nullen; für den aktuellen und den nächsten Track ersetzt ein Symbol sie |
| Titel | Aus den Tags oder dem Dateinamen (`Artist - Title.mp3` wird aufgeteilt). Die Wiederholen- und Stopp-danach-Symbole eines Tracks stehen davor |
| Interpret | Aus den Tags; „Unbekannter Interpret“, wenn es keinen gibt |
| Album | Aus den Tags |
| Datum | Das Aufnahmedatum, wie die Datei es speichert (`2019`, `2019-05` oder `2019-05-14`, mit Uhrzeit, falls vorhanden) |
| Genre | Aus den Tags |
| Dauer | Spieldauer, vom Cue-in bis zum Cue-out (die ganze Datei, wenn **Cue-in und Cue-out verwenden** ausgeschaltet ist) |
| Intro | Wie lange das Intro dauert, von der Stelle, an der der Track zu spielen beginnt, bis zu seinem Intro-Marker; leer, wenn der Track keinen Intro-Marker hat |
| Dateiname | Der Name der Datei, mit ihrer Endung |

Eine Neuinstallation zeigt `#`, Titel, Interpret und Dauer. Die anderen Spalten
sind optional; siehe **Die Spalten wählen** weiter unten. Fehlt einem Track ein
Wert, zeigt die Zelle nichts an, außer bei Interpret, das „Unbekannter
Interpret“ zeigt.

Die Spalten füllen die Tabelle und behalten ihre Proportionen, wenn die Größe
des Fensters geändert wird; die Textspalten bekommen den meisten Platz. Ziehe
die Trennlinien der Kopfzeile, um die Proportionen zu ändern: Die Spalten
rechts der Trennlinie folgen dem Zeiger in jedem Frame (sie teilen sich, was
übrig ist, im Verhältnis ihrer Breite), die links davon bleiben, und die
Breiten werden beim Loslassen gespeichert. Keine Spalte wird schmaler als ihr
Minimum. Die Breiten werden pro Player gemerkt; eine Spalte, die du später
einblendest, beginnt mit ihrer Standardbreite, und die anderen behalten ihre
Proportionen.

### Die Spalten wählen {#choosing-the-columns}

Titel und Dauer werden immer angezeigt. Jede andere Spalte kann ein- oder
ausgeblendet werden, und jede Spalte, auch diese beiden, kann verschoben
werden. Die Liste ist für jeden Player und jede Playlist dieselbe und wird in
`config.json` als `ui.table_columns` gespeichert. Drei Wege, sie zu ändern:

- **Einstellungen → Playlists → Tabellenspalten:** Setze Haken bei den Spalten,
  die angezeigt werden sollen; die Pfeile verschieben eine angezeigte Spalte
  nach oben oder unten (sie werden in den Tabellen von links nach rechts
  gelesen). **Standardspalten** kehrt zu `#`, Titel, Interpret und Dauer
  zurück.
- **Rechtsklick auf eine Kopfzeile:** ein Menü mit einem Kontrollkästchen für
  jede optionale Spalte. Eine Spalte, die du einblendest, erscheint am rechten
  Ende; ziehe sie von dort.
- **Eine Kopfzeile ziehen** und auf eine andere legen: Lege sie auf die linke
  Hälfte einer Kopfzeile, um die Spalte davor einzufügen, auf die rechte
  Hälfte, um sie danach einzufügen. Überall sonst abzulegen bewirkt nichts.

Ein Name in `ui.table_columns`, den diese Version nicht kennt, wird ignoriert,
und ein fehlender Titel oder fehlende Dauer wird wieder hinzugefügt.

Wenn die Anwendung geöffnet wird, scrollt jede Tabelle so, dass der nächste
Track ihres Players in der Mitte der Tabelle liegt (so nah, wie die Enden der
Liste es zulassen). Das geschieht einmal, beim Start, und nur, wenn der
nächste Track in der Playlist liegt, die die Tabelle zeigt.

Wechselt ein Player zu einem anderen Track, zeigt seine Tabelle die Playlist
dieses Tracks und scrollt seine Zeile nach oben, außer du hast die Tabelle in
den letzten 10 Sekunden benutzt (gescrollt, einen Track gezogen, das Menü eines
Tracks geöffnet oder auf einen Tab geklickt): Dann wartet sie, bis du sie so
lange in Ruhe gelassen hast. Die Zeit ist `ui.follow_current_grace_secs` in
`config.json`; 0 schaltet das Folgen aus.

Die Player sind unabhängig: Mehrere Player können dieselbe Playlist anzeigen,
jeder mit seinem eigenen nächsten Track, seinen eigenen Gespielt-Markierungen
und seinen eigenen Zeiten in der Fußzeile. Abspielen, Stoppen oder Überspringen
an einem Player verschiebt nie den nächsten Track eines anderen Players.
Derselbe Track kann sogar auf zwei Playern gleichzeitig on Air sein. Das
Bearbeiten der Playlist (Hinzufügen, Verschieben oder Entfernen von Einträgen)
oder eine Datei, die unlesbar wird, kann dennoch den nächsten Track jedes
Players ändern, der sie anzeigt.

Zeilenfarben:

| Zeile | Bedeutung |
|---|---|
| **Rot**, mit einem Lautsprecher- (oder Pause-)Symbol | On Air auf diesem Player. Sie kann auch den grünen Pfeil zeigen: Der Track, der on Air ist, ist auch der nächste, spielt also noch einmal |
| Rotes **P2** (oder eine andere Nummer) in der Nummernspalte | On Air auf jenem Player |
| **Grün**, mit einem Pfeil | Der nächste Track dieses Players |
| Abgeblendet | Auf diesem Player bereits gespielt |
| Datei mit Kreuz / Warnsymbol | Datei fehlt / ist unlesbar (sie wird übersprungen); fahre mit der Maus über die Zeile: Das Popup beginnt mit dem Grund, in Bernstein, dann folgen die üblichen Felder. Eine fehlende Datei wird alle 30 s erneut gesucht (`tuning.missing_recheck_ms`). |
| Neu-laden-Pfeile rechts vom Titel | Von einer älteren Version analysiert; er spielt weiterhin mit dieser Analyse. **Einstellungen → Analyse → Veraltete Tracks analysieren** bringt ihn auf den neuesten Stand (Tracks auf einem Player werden ohnehin aktualisiert) |
| Sanduhr rechts vom Titel | Der Track wartet auf seine Analyse (fahre mit der Maus über die Sanduhr: *Analyse ausstehend*). Er spielt trotzdem, und die Sanduhr verschwindet, wenn die Analyse fertig ist |
| Violett | Ausgewählt |

**Track-Tooltip.** Verweile einen Moment mit der Maus auf einer Zeile, um Titel, Interpret, Album, Datum, Genre, Länge, Format (Typ, Abtastrate und Bittiefe, wenn bekannt) und den Pfad ihrer Datei zu sehen. Ein Feld, das die Datei nicht hat, wird weggelassen. Das Popup ist die einzige Hover-Information an einer Zeile, und es bewegt sich nie, solange es angezeigt wird. Bei einer fehlenden oder unlesbaren Datei beginnt es mit dem Grund.

## Maus {#mouse}

- **Klick** wählt einen Track aus. **Doppelklick** macht ihn zum nächsten Track
  dieses Players. Beim Track, der on Air ist, bewirkt er, dass dieser noch
  einmal spielt, wenn der aktuelle Durchlauf endet.
- **Rechtsklick** öffnet das Kontextmenü:

![Das Kontextmenü eines Tracks](../../images/guide/track-menu.png)

| Eintrag | Aktion |
|---|---|
| Jetzt abspielen | Diesen Track sofort starten (mit Mix, wenn der Player on Air ist) |
| Als nächsten setzen | Wie Doppelklick. Beim Track, der on Air ist, spielt er ab seinem Anfang noch einmal, wenn der aktuelle Durchlauf endet (mit Mix wie bei Wiederholen, ohne Lücke), dann geht der Player weiter. Es wirkt einmalig. Stopp danach, der SINGLE-Modus und eine Stopp-danach-Markierung beenden den Player dennoch zuerst. Läuft ein CUE, wechselt es zum neuen nächsten Track |
| Auf CUE vorhören | Auf dem CUE-Ausgang abspielen (es öffnet das CUE-Fenster). Abgeblendet, wenn der Player neben seinem Main-Ausgang keinen Cue-Ausgang hat |
| Tags bearbeiten… | Den Tag-Editor für diesen Track öffnen. **Speichern** schreibt die Änderungen in die Audiodatei; **Abbrechen** (oder Esc, wenn kein Speichern läuft) schließt ohne zu schreiben. Der Eintrag ist abgeblendet, mit dem Grund beim Darüberfahren, solange der Track on Air ist, auf CUE oder auf einem laufenden Cart, solange seine Tags noch nicht gelesen wurden, wenn die Datei fehlt und bei Formaten, deren Tags sich nicht schreiben lassen (zum Beispiel DSD) |
| Neu analysieren | Diesen Track jetzt noch einmal analysieren, egal in welchem Zustand. Eine reparierte Datei, die zuvor unlesbar war, wird auch von selbst erkannt (siehe [Fehlerbehebung](troubleshooting.md)). Manuelle Marker bleiben erhalten |
| Tracks darunter hinzufügen… | Dateien wählen, die nach diesem Track eingefügt werden |
| Duplizieren | Eine ungespielte Kopie darunter einfügen (mit ihren Wiederholen- und Stopp-danach-Markierungen) |
| Diesen Track wiederholen | Haken setzen, um ihn immer wieder ohne Lücke zu spielen, bis du Play (Weiter), Zurück, Stopp oder Ausblenden drückst oder Stopp danach einschaltest. Pause lässt ihn weiter wiederholen. Vor dem Titel erscheint ein Wiederholen-Symbol |
| Nach diesem Track stoppen | Haken setzen, um den Player zu stoppen, wenn dieser Track endet, jedes Mal, wenn er gespielt wird (in jedem Modus). Anders als der Button **Nach dem aktuellen Track stoppen** des Players bleibt die Markierung am Track und wird mit der Playlist gespeichert. Vor dem Titel erscheint das Stopp-danach-Symbol. Es gewinnt gegenüber Wiederholen |
| Verschieben nach ▸ | Ans Ende einer anderen Playlist verschieben |
| Aus der Playlist entfernen | Entfernen; nicht möglich, solange er on Air ist |

## Tags bearbeiten {#editing-tags}

**Tags bearbeiten…** öffnet ein Fenster für einen Track. Solange es offen ist,
wirkt kein Tastenkürzel, und auf das Anwendungsfenster gezogene Dateien werden
ignoriert.

![Der Tag-Editor einer FLAC-Datei, mit Cover, Titel, Interpret, Album, Datum und Genre](../../images/guide/tag-editor.png)

- **Was du siehst.** Der Editor liest die Datei, wenn er sich öffnet (währenddessen
  zeigt er „Tags werden gelesen…“). Immer angezeigt: Titel, Interpret, Album,
  Album-Interpret, Datum, Tracknummer und Gesamtzahl, CD-Nummer und Gesamtzahl,
  Genre, Komponist und Kommentar. Angezeigt, wenn die Datei sie hat: Untertitel,
  Gruppierung, BPM, Tonart, Stimmung, ISRC, Verlag, Katalognummer, Copyright,
  Originalinterpret, Originalalbum, ursprüngliches Erscheinungsdatum, Texter,
  Dirigent, Remixer, Arrangeur, Ausführender, Sprache, Kodiert von, Liedtext,
  Sortiertitel, Sortierinterpret, Sortieralbum, Sortier-Album-Interpret,
  Sortierkomponist und Website des Interpreten.
- **Feld hinzufügen.** Das Menü unter den Feldern listet die übrigen Felder auf.
  Es bietet nur an, was das Tag-Format der Datei speichern kann (ein WAV mit
  RIFF INFO, ein AIFF oder ein altes ID3v1-Tag speichern weniger Felder als
  ID3v2, FLAC oder MP4), und es ist abgeblendet, wenn nichts mehr hinzuzufügen
  ist. Eines der immer angezeigten Felder, das das Format nicht speichern kann,
  ist mit einem Hinweis ausgegraut. Ein Feld zu leeren entfernt es aus der
  Datei; ein hinzugefügtes Feld, das leer bleibt, wird nicht geschrieben.
- **Mehrere Werte.** Felder, die mehrere Werte aufnehmen können (Interpret,
  Album-Interpret, Genre, Komponist, Stimmung und die Credits wie Texter,
  Dirigent, Remixer, Arrangeur und Ausführender sowie Sprache), zeigen einen
  Wert pro Zeile; **Speichern** schreibt einen Wert pro Zeile auf die Art des
  Formats. Kommentar und Liedtext sind Freitext über mehrere Zeilen.
- **Prüfungen.** Datum und ursprüngliches Erscheinungsdatum sind ISO 8601
  (`2019`, `2019-05` oder `2019-05-14`, optional mit Uhrzeit); Track- und
  CD-Nummer, ihre Gesamtzahlen und die BPM sind ganze Zahlen, und eine
  Gesamtzahl braucht ihre Nummer. Ein Feld mit ungültigem Wert wird markiert,
  und **Speichern** bleibt ausgeschaltet. Ein Wert, den die Datei schon hatte
  und den du nicht angefasst hast, bleibt unverändert.
- **Zu lange Felder.** Ein Feld, dessen Text länger als `limits.max_tag_chars`
  ist oder das mehr Werte als `limits.max_tag_values` enthält, wird schreibgeschützt
  mit dem Hinweis „Zu lang, um es hier zu bearbeiten; bleibt in der Datei
  unverändert“ angezeigt. Es wird nie zurückgeschrieben, sodass ein Speichern
  es nicht abschneiden kann.
- **Was erhalten bleibt.** Alles, was der Editor nicht zeigt (andere
  Standardschlüssel, benutzerdefinierte Schlüssel, andere Bilder als das
  Frontcover, binäre Frames), bleibt mit denselben Werten in der Datei. Der
  Editor sagt, wie viele solche Tags erhalten bleiben (und „weitere“, wenn das
  Format Frames enthält, die sich nicht zählen lassen). Beim Speichern werden
  die Einträge, die der Editor abbildet, neu kodiert, sodass ein erhaltener
  Eintrag in seinen Bytes abweichen kann (Textkodierung, Frame-Reihenfolge),
  nicht aber in seinem Wert.
- **Das Cover.** Der Editor zeigt das Frontcover oder, wenn es kein Frontcover
  gibt, das erste Bild der Datei als Vorschaubild.
  - **Ändern…** öffnet einen Dateidialog für ein JPEG- oder PNG-Bild (höchstens
    `limits.max_cover_bytes`, und es muss sich dekodieren lassen). Ist das nicht
    der Fall, nennt der Editor den Grund, und nichts ändert sich.
  - **Entfernen** löscht das Frontcover. Es ist ausgeschaltet, wenn die Datei
    kein Frontcover hat: Ein Bild, das nur angezeigt wird, weil es kein
    Frontcover gibt, dient nur der Anzeige und bleibt unverändert.
  - Ein Cover, das in der Datei ist, sich aber nicht anzeigen lässt (ein Bild,
    das sich nicht dekodieren lässt, oder ein GIF, BMP oder WebP), wird mit
    „Dieses Cover kann nicht angezeigt werden; es bleibt unverändert“ gemeldet.
    **Ändern…** und **Entfernen** funktionieren weiterhin.
  - Die Änderung wird durch **Speichern** geschrieben und durch **Abbrechen**
    verworfen. Rückseitencover und alle anderen Bilder werden nie angefasst. Ein
    Format ohne Platz für Bilder (WAV mit RIFF INFO, AIFF, ID3v1) zeigt den
    Bereich deaktiviert. Nach einem Speichern zeigt das Cover des Players das
    neue Cover.
- **Wie ein Speichern funktioniert.** Die Datei wird neben das Original kopiert,
  die Kopie erhält die Tags, wird synchronisiert und ersetzt das Original,
  sodass ein Fehler die Datei so lässt, wie sie war. Der Grund erscheint im
  Editor, der zum erneuten Versuch offen bleibt, und in der Statusleiste. Es
  werden nur die Felder geschrieben, die du geändert hast. Nach einem Speichern
  zeigt die Tabelle sofort die neuen Tags, und Marker und Wellenform bleiben
  erhalten. Hat die Datei ein von dir geändertes Feld nicht übernommen, nennt
  die Statusleiste es.
- **Nach einem Update.** Bei Tracks einer früheren Version werden Datum, Genre
  und andere Tags im Hintergrund still ergänzt (keine vollständige Analyse).

## Drag and Drop {#drag-and-drop}

- Ziehe einen Track innerhalb der Liste, um ihn umzusortieren. Eine violette
  Linie zeigt, wo er landen wird: an der Zeilengrenze, die dem Zeiger am
  nächsten ist, und nur in der Liste unter dem Zeiger. Loslassen über der
  Kopfzeile, einem Spaltenrand, der Bildlaufleiste oder einem Fenster, das die
  Liste verdeckt (das CUE-Fenster), legt nichts ab.
- Halte den Zeiger beim Ziehen eines Tracks nahe an den oberen oder unteren Rand
  einer Liste, um sie zu scrollen: Je näher am Rand, desto schneller geht es,
  und es endet an den Enden der Liste oder wenn du dich vom Rand entfernst. Das
  Mausrad scrollt die Liste auch während des Ziehens. Die violette Linie folgt
  weiter dem Zeiger, während sich die Liste bewegt. Das Ziehen von Dateien aus
  dem Dateimanager über eine Liste scrollt sie genauso, wo das System die
  Zeigerposition meldet, sobald du den Zeiger über die Liste bewegst.
- Ziehe ihn auf die Liste eines anderen Players, um ihn dorthin zu verschieben.
- Ziehe ihn auf einen Tab, um ihn an diese Playlist anzuhängen.
- Lege Dateien oder Ordner aus dem Dateimanager auf einer Liste ab, um sie an
  der Ablageposition einzufügen. Über der Kopfzeile, einem Spaltenrand, der
  Bildlaufleiste oder einem Fenster, das die Liste verdeckt, wird nichts
  eingefügt. Meldet das System die Position nicht, landen sie am Ende der
  angezeigten Liste.

## Fußzeile {#footer}

**+ Hinzufügen** öffnet einen Dateidialog, der im in den Einstellungen
festgelegten Musikordner beginnt. **Gespielt zurücksetzen** (das Pfeilsymbol
daneben) löscht für jeden Player die abgeblendete Markierung „bereits gespielt“
aller Tracks der Playlist, nachdem es gefragt hat: „Die Gespielt-Markierung
aller Tracks dieser Playlist löschen?“ (**Abbrechen**, Esc oder ein Klick
außerhalb behalten die Markierungen). Der Track, der on Air ist, behält seinen
Zustand und wird markiert, wenn der Player ihn verlässt. Der Button ist
abgeblendet, wenn es nichts zu löschen gibt. Die Fußzeile zeigt außerdem die
Anzahl der Tracks, die Restzeit der Playlist und ihre Gesamtlänge.

## Playlist-Dateien {#playlist-files}

- **Import:** Einstellungen → Playlists → **M3U / PLS importieren…**, oder eine
  `.m3u`-, `.m3u8`- oder `.pls`-Datei auf das Fenster ziehen. Sie wird zu einer
  neuen Playlist, benannt nach der Datei.
  - Relative Pfade werden relativ zum Ordner der Playlist-Datei aufgelöst.
  - `file://`-Adressen werden verstanden.
  - Dateien, die nicht gefunden werden, werden trotzdem hinzugefügt und als
    nicht verfügbar markiert.
  - Internet-Streams werden übersprungen; eine Meldung nennt, wie viele.
- **Export:** Der Button **M3U** an jeder Playlist in den Einstellungen
  speichert sie als M3U8-Datei mit Titeln, Längen und vollständigen Pfaden.
