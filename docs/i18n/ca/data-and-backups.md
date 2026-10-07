# Dades i còpies de seguretat

## On es desen els fitxers {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Configuració (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Llistes i sessió | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Memòria cau d'anàlisi | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Registres i informes de fallades | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Els camins de Linux segueixen les variables XDG (`XDG_CONFIG_HOME` i
similars) quan estan definides.

## Mode portàtil {#portable-mode}

Defineix la variable d'entorn `FAUSTE_HOME` amb una carpeta, i tot es desa
allà, a `config/`, `data/`, `cache/` i `logs/`. És útil en una memòria USB, o
per mantenir configuracions separades una al costat de l'altra.

## Fitxers {#files}

| Fitxer | Contingut |
|---|---|
| `config.json` | Configuració, sortides, llindars d'anàlisi, ajustos avançats |
| `playlists.json` | Llistes, pistes, marques de reproduïda, marcadors manuals |
| `session.json` | Per a cada reproductor: llista mostrada, pistes actual i següent, mode, posició, volum, amplades de columna (per columna) |

## Desament automàtic i còpies de seguretat {#autosave-and-backups}

- Els canvis es desen aproximadament un segon després que passin. Mentre
  alguna cosa sona, la sessió (posicions) s'actualitza al mateix ritme.
- Cada desament escriu un fitxer temporal i després substitueix el vell, de
  manera que un tall de corrent mai no deixa un fitxer escrit a mitges.
- Les versions anteriors es conserven com a `.bak1`, `.bak2` i `.bak3`.
- Si un fitxer no es pot llegir, s'usa la còpia bona més recent. El fitxer
  il·legible es conserva, amb el nom canviat a `*.corrupt-<hora>`, per
  inspeccionar-lo. L'aplicació sempre s'inicia.

## Recuperació després d'una fallada {#crash-recovery}

Després d'una fallada o d'un reinici, cada reproductor torna amb la seva
llista, les seves pistes actual i següent, i la seva posició, però **en pausa
o aturat**. Res no surt en antena per si sol. Un reproductor que era al final
mateix de la seva pista torna a l'inici d'aquella pista, de manera que prémer
Play la reprodueix en lloc d'acabar-la a l'instant.

## Editar `config.json` a mà {#editing-configjson-by-hand}

Tanca primer l'aplicació (si hi ha àudio en antena, ho pregunta abans de
tancar). Els valors desconeguts o fora d'interval es corregeixen al valor
vàlid més proper quan es carrega el fitxer, i les correccions es registren.
Les seccions `limits` i `tuning` contenen valors avançats (límits de recursos,
temporització del motor) que no són a la finestra de Configuració.
