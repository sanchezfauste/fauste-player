# Gegevens en back-ups

## Waar bestanden staan {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Instellingen (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Playlists en sessie | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Analysecache | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Logbestanden en crashrapporten | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

De Linux-paden volgen de XDG-variabelen (`XDG_CONFIG_HOME` enzovoort) wanneer
die zijn ingesteld.

## Portable-modus {#portable-mode}

Stel de omgevingsvariabele `FAUSTE_HOME` in op een map, en alles wordt daar
bewaard, in `config/`, `data/`, `cache/` en `logs/`. Dat is handig op een
USB-stick, of om aparte opstellingen naast elkaar te houden.

## Bestanden {#files}

| Bestand | Inhoud |
|---|---|
| `config.json` | Instellingen, uitgangen, analysedrempels, geavanceerde afstelling |
| `playlists.json` | Playlists, nummers, gespeeld-markeringen, handmatige markers |
| `session.json` | Per speler: getoonde playlist, huidig en volgend nummer, modus, positie, volume, kolombreedtes (per kolom) |

## Automatisch opslaan en back-ups {#autosave-and-backups}

- Wijzigingen worden ongeveer een seconde nadat ze plaatsvinden opgeslagen.
  Terwijl er iets speelt, wordt de sessie (posities) in hetzelfde tempo
  vernieuwd.
- Elke keer opslaan schrijft eerst een tijdelijk bestand en vervangt daarmee
  het oude, zodat een stroomstoring nooit een half geschreven bestand
  achterlaat.
- De vorige versies blijven bewaard als `.bak1`, `.bak2` en `.bak3`.
- Kan een bestand niet worden gelezen, dan wordt de nieuwste goede back-up
  gebruikt. Het onleesbare bestand blijft bewaard, hernoemd naar
  `*.corrupt-<tijd>`, om te onderzoeken. De applicatie start altijd.

## Herstel na een crash {#crash-recovery}

Na een crash of een herstart komt elke speler terug met zijn playlist, zijn
huidige en volgende nummer en zijn positie, maar **gepauzeerd of gestopt**. Er
gaat niets uit zichzelf on air. Een speler die helemaal aan het einde van zijn
nummer was, komt in plaats daarvan terug aan het begin van dat nummer, zodat
Play het afspeelt in plaats van het meteen te beëindigen.

## `config.json` met de hand bewerken {#editing-configjson-by-hand}

Sluit eerst de applicatie (als er audio on air is, vraagt ze dat eerst te
bevestigen). Onbekende of buiten het bereik vallende waarden worden bij het
laden van het bestand gecorrigeerd naar de dichtstbijzijnde geldige waarde, en
de correcties worden gelogd. De secties `limits` en `tuning` bevatten
geavanceerde waarden (resourcelimieten, engine-timing) die niet in het venster
Instellingen staan.
