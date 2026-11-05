# Fernsteuerung

Fauste Player lässt sich über das Netzwerk durch eine HTTP-API mit
Live-Updates und über OSC auslesen und bedienen. Eine Webseite, eine
Smartphone-App, die Automatisierung eines Senders oder eine Bedienoberfläche
können sie nutzen. Sie ist **aus**, bis du sie einschaltest, und antwortet
zunächst nur auf diesem Computer.

## Einschalten {#turning-it-on}

![Einstellungen, Fernsteuerung: die HTTP-API an und auf diesem Computer lauschend, und OSC aus](../../images/guide/settings-remote.png)

Öffne **Einstellungen → Fernsteuerung** und setze den Haken bei
**Fernsteuerung über HTTP erlauben** (oder **Steuerung über OSC erlauben**). Die
Zeile unter jedem Schalter sagt, ob der Server lauscht und wo, oder warum er
nicht gestartet ist. Änderungen gelten sofort; ein Neustart ist nicht nötig. Ein
Textfeld (eine Adresse, das Token, eine Liste) gilt, wenn du es verlässt, einen
anderen Abschnitt öffnest oder die Einstellungen schließt; ein Wert, der noch
nicht gültig ist, behält den in Verwendung befindlichen, und Esc bricht ab, was
du getippt hast.

Du kannst auch `config.json` bearbeiten, während Fauste Player geschlossen ist
(siehe [Daten und Backups](data-and-backups.md), wo sie liegt). Setze im
`"config"`-Objekt `remote.http.enabled` auf `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Sie lauscht auf `http://127.0.0.1:7380`. Ihr Start spielt nie etwas ab; nur
Anfragen wirken.

## Im Studionetzwerk lauschen {#listening-on-the-studio-network}

Um sie von anderen Computern aus zu erreichen, setze `bind` auf `0.0.0.0` (oder
eine der Adressen dieses Computers) und lege ein **Token** mit mindestens 16
Zeichen fest. In Einstellungen → Fernsteuerung erzeugt **Erzeugen** ein langes
zufälliges Token. Es ist verborgen, bis du **Anzeigen** drückst, und **Kopieren**
legt es für den Client in die Zwischenablage. Ohne Token weigert sich der Server
zu starten, und das Log sagt, warum.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Clients senden das Token als `Authorization: Bearer <token>`. Die API ist nicht
verschlüsselt. Betreibe sie in einem vertrauenswürdigen Studionetzwerk oder
stelle sie hinter einen Reverse-Proxy mit HTTPS.

## Webseiten {#web-pages}

Eine Webseite, die von einer anderen Adresse ausgeliefert wird, kann die API nur
nutzen, wenn ihr Origin (zum Beispiel `https://studio.example`) in
`cors_origins` aufgeführt ist. Anfragen anderer Seiten werden abgelehnt, auch auf
diesem Computer, sodass eine Seite, die du zufällig offen hast, den Player nicht
steuern kann. `"*"` (jeder Origin) wird nur zusammen mit einem Token akzeptiert.

## Was ein Client tun kann {#what-a-client-can-do}

Ein Client kann:

- die Player, Playlists, Tracks (mit Cover und Wellenform) und die Cartwall
  auslesen;
- abspielen, pausieren, stoppen, ausblenden, neu starten und zurückgehen;
- den nächsten Eintrag wählen (auch den Eintrag on Air: er spielt noch einmal),
  vorhören (ein Vorhören, das auf Sendung zu hören wäre, während dort etwas
  spielt, wird abgelehnt, es sei denn, die Anfrage enthält `"force": true`; siehe
  die API-Referenz) und springen (bei einem gestoppten Player wählt ein Sprung, wo Play
  den nächsten Eintrag startet, und ein Sprung vor sein Cue-in startet am
  Cue-in);
- Lautstärken, Modi, Stopp nach dem aktuellen Track sowie die Wiederholen- und
  Stopp-danach-Markierungen eines Eintrags setzen;
- Carts starten, stoppen und vorhören und die angezeigte Cart-Seite wechseln;
- bearbeiten: Playlists erstellen, umbenennen und löschen; einen bereits
  geladenen Track hinzufügen und Einträge entfernen, verschieben oder
  duplizieren; Cart-Seiten erstellen, umbenennen, in der Größe ändern und
  löschen und ein Cart (Name, Typ, Farbe, Schleife, exklusiv) mit einem geladenen
  Track einrichten; Marker setzen oder zurücksetzen; einen Track von der
  [Lautheitsnormalisierung](loudness.md) ausnehmen oder seinen Trim setzen.

Das Entfernen dessen, was on Air ist, wird abgelehnt, wie auf dem Bildschirm.
Dateien, die noch nicht geladen sind, lassen sich nicht per Fernsteuerung
hinzufügen: Sie liegen auf diesem Computer, füge sie also zuerst hier hinzu.

Ein Button, der auf dem Bildschirm ausgegraut ist, wird auch per Fernsteuerung
abgelehnt.

Ein Player, dessen Main-Ausgang [bit-perfect](bit-perfect.md#what-it-costs) ist,
funktioniert per Fernsteuerung weiter, mit denselben Grenzen wie auf dem
Bildschirm: `play` und `previous` wechseln sofort, ohne Crossfade, und seine
Lautstärke ist auf 100 % festgehalten. Eine Lautstärke unter 100 % wird abgelehnt
(`409`, mit dem Grund), genau 100 % wird akzeptiert. Jeder Player sagt, ob seine
Lautstärke festgehalten ist (`fader_locked`).

Eine als Fader-Position gesendete Lautstärke (das `fader` der API, OSC `/volume`)
ist eine Position auf der eigenen Skala des Faders auf dem Bildschirm, von 0
(Stille) bis 1 (0 dB): 0.9 sind etwa −2,5 dB, 0.8 etwa −5,3 dB und 0.5 etwa
−16 dB. Frühere Versionen verwendeten eine andere Skala (0.8 waren −12 dB, 0.5
waren −30 dB), sodass ein dafür geschriebenes Skript jetzt lautere Pegel setzt:
Prüfe seine Werte.

Jeder Track trägt seine Lautheitsmessung (`loudness`, `null`, bis er gemessen ist)
und seine Einstellung (`loudness_edit`: `excluded` und `trim_db`). Ein spielender
oder pausierter Player und jedes spielende Cart sagt, mit welcher
[Normalisierungs](loudness.md)-Verstärkung sein Track gestartet ist und warum
(`normalization`: `gain_db` und ein `status` wie `applied`, `not_measured` oder
`bit_perfect`). `PUT /api/v1/tracks/<id>/loudness` mit
`{"excluded": false, "trim_db": -1.5}` ändert die Einstellung, wie es das Popup
**Lautheit…** tut; ein Trim über dem maximalen Trim hinaus wird abgelehnt (`409`,
mit dem Grund). Die Normalisierungseinstellungen selbst (an oder aus, das Ziel,
die Grenzen) werden nur auf diesem Computer geändert. OSC und MIDI haben keine
Lautheitsregler.

Die vollständige Referenz steht in [der technischen
Dokumentation](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Probiere es in einem Terminal aus:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Live-Updates {#live-updates}

Ein Client kann Änderungen verfolgen, während sie geschehen, statt immer wieder
nachzufragen. `GET /api/v1/events` ist ein Strom von Ereignissen: zuerst der
ganze Zustand, dann jede Änderung eines Players, einer Playlist, eines Tracks
oder der Cartwall und mehrmals pro Sekunde die Zeiten dessen, was spielt.

    curl -sN http://127.0.0.1:7380/api/v1/events

Eine Webseite verwendet `EventSource`. Browser können dort das Token nicht als
Header senden, daher steht es in der Adresse:
`/api/v1/events?token=<token>`.

## OSC {#osc}

OSC ist das übliche Protokoll von Bedienoberflächen, Lichtpulten und
Show-Control-Software. Schalte es mit `remote.osc.enabled` ein. Es lauscht auf dem
UDP-Port 7381 dieses Computers. Um Pakete anderer Computer anzunehmen, setze
`remote.osc.bind` auf `0.0.0.0` und liste ihre Adressen oder Subnetze in
`remote.osc.allowed_sources` auf (zum Beispiel `"192.168.1.0/24"`). OSC hat kein
Passwort, betreibe es also in einem vertrauenswürdigen Studionetzwerk.

Die Player sind so nummeriert, wie sie auf dem Bildschirm erscheinen: 1, 2, 3…
Carts sind in der angezeigten Seite nummeriert.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Eine Oberfläche, die den Zustand anzeigen will (Lichter, Namen, Countdowns),
abonniert und erhält dann jeden Wert einmal und danach nur noch, was sich
ändert. Verschwindet ein Player oder ein Cart-Button (weniger Player, eine
kleinere Seite), erhalten seine Adressen einmal einen leeren Wert, damit die
Oberfläche sie löscht. Sie muss sich innerhalb einer Minute
(`subscription_ttl_secs`) erneut anmelden, um weiter zu empfangen:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Bei einem Player, dessen Main-Ausgang bit-perfect ist, werden
`/volume`-Nachrichten abgelehnt (das Log vermerkt es), und der Player sendet
`/volume-locked` 1 an die Abonnenten (sonst 0).

Ein Abonnent kann jeden Port seiner eigenen Adresse nennen, und bis zu
`max_subscribers` werden behalten. Jeder, der senden darf, kann daher auch
abonnieren. Das ist ein weiterer Grund, OSC in einem vertrauenswürdigen Netzwerk
zu betreiben.

`oscsend` und `oscdump` gehören zu liblo (`liblo-tools` unter Debian und
Ubuntu). Die vollständige Liste der Adressen steht in
[der technischen Dokumentation](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

## Alle Einstellungen {#all-settings}

| Einstellung | Standard | Bedeutung |
|---|---|---|
| `remote.http.enabled` | `false` | Die API einschalten |
| `remote.http.bind` | `127.0.0.1` | Adresse, auf der gelauscht wird (eine IP-Adresse) |
| `remote.http.port` | `7380` | Port (1024–65535) |
| `remote.http.token` | leer | Über diesen Computer hinaus erforderlich; mindestens 16 Zeichen |
| `remote.http.cors_origins` | keine | Web-Origins, die die API aufrufen dürfen |
| `remote.http.request_timeout_ms` | `10000` | Wie lange eine Anfrage höchstens dauern darf |
| `remote.http.max_body_bytes` | `65536` | Größter Anfragekörper |
| `remote.http.max_event_clients` | `16` | Gleichzeitige Live-Ereignisströme |
| `remote.osc.enabled` | `false` | OSC einschalten |
| `remote.osc.bind` | `127.0.0.1` | Adresse, auf der gelauscht wird |
| `remote.osc.port` | `7381` | UDP-Port (1024–65535) |
| `remote.osc.allowed_sources` | dieser Computer | Adressen oder Subnetze, deren Pakete angenommen werden |
| `remote.osc.max_subscribers` | `16` | Gleichzeitige Abonnenten |
| `remote.osc.subscription_ttl_secs` | `60` | Ein Abonnement, das nicht innerhalb dieser Zeit erneuert wird, endet |
| `remote.events.position_interval_ms` | `250` | Wie oft Zeiten während der Wiedergabe gesendet werden |

Werte außerhalb des Bereichs werden beim Laden der Datei korrigiert, und die
Korrektur wird protokolliert.
