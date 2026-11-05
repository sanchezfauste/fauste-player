# Dades i còpies de seguretat

## On es desen els fitxers {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Configuració (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Biblioteca, llistes, cartutxos i sessió (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Memòria cau d'anàlisi | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Registres i informes de fallades | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Els camins de Linux segueixen les variables XDG (`XDG_CONFIG_HOME` i
similars) quan estan definides.

El snap desa els seus fitxers en una carpeta pròpia: la configuració a
`~/snap/fauste-player/current/.config/fausteplayer/`, la base de dades i els
registres a `~/snap/fauste-player/current/.local/share/fausteplayer/`, i la
memòria cau d'anàlisi a `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` també els esborra (conserva una instantània
durant un temps; `--purge` no).

## Mode portàtil {#portable-mode}

Defineix la variable d'entorn `FAUSTE_HOME` amb una carpeta, i tot es desa
allà, a `config/`, `data/`, `cache/` i `logs/`. És útil en una memòria USB, o
per mantenir configuracions separades una al costat de l'altra.

## Fitxers {#files}

| Fitxer | Contingut |
|---|---|
| `config.json` | Configuració, sortides, llindars d'anàlisi, ajustos avançats |
| `fauste.db` | La biblioteca (pistes, les seves categories, marcadors manuals, etiquetes, sonoritat), les llistes amb les seves marques de reproduïda, les pàgines de cartutxos i la sessió: per a cada reproductor la llista mostrada, les pistes actual i següent, el mode, la posició, el volum i les amplades de columna (per columna), i la disposició de la finestra |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Còpies de la base de dades (vegeu més avall) |
| `window.ron` | La mida i la posició de la finestra principal i si estava maximitzada (només a l'ordinador; a Wayland el sistema tria la posició) |

## Desament automàtic i còpies de seguretat {#autosave-and-backups}

- Els canvis es desen aproximadament un segon després que passin, i només
  s'escriu el que ha canviat. Mentre alguna cosa sona, la sessió (posicions)
  s'actualitza al mateix ritme.
- La base de dades s'escriu en transaccions, de manera que un tall de corrent
  mai no deixa un canvi desat a mitges.
- Al primer desament de cada sessió, i després cada cinc minuts més o menys
  mentre es desen canvis, l'aplicació copia la base de dades a `fauste.db.bak1`
  i desplaça les còpies més velles cap amunt, a `.bak2` i `.bak3`. Reiniciar
  diverses vegades seguides, doncs, treu les còpies més velles.
- Si la base de dades no es pot llegir, s'usa la còpia bona més recent. El
  fitxer il·legible es conserva, amb el nom canviat a `fauste.db.corrupt-<hora>`,
  per inspeccionar-lo. Si totes les còpies fallen, l'aplicació s'inicia buida.
  Sempre s'inicia.
- Una còpia que no es pot usar també es conserva: quan l'aplicació s'inicia
  des d'una altra còpia (o d'una base de dades buida), les inutilitzables es
  reanomenen `fauste.db.bak1.corrupt-<hora>` (o `.newer-<hora>`) perquè la
  còpia següent no les tregui mai.
- Si una altra còpia de l'aplicació està usant la base de dades, aquesta
  s'executa sense desar i ho diu a l'inici.
- Una base de dades escrita per una versió més nova de l'aplicació no
  s'obre: es conserva com a `fauste.db.newer-<hora>`, i l'aplicació s'inicia
  des de la còpia més recent que pot llegir, o buida si no n'hi ha cap, de
  manera que obrir les dades amb una versió més antiga mai no les destrueix.
- `config.json` es desa de la mateixa manera que abans: un fitxer temporal
  substitueix el vell, i les versions anteriors es conserven com a `.bak1`,
  `.bak2` i `.bak3` al seu costat.
- Si la base de dades no es pot usar en absolut en una sessió (per exemple,
  no es pot reanomenar perquè la carpeta és de només lectura), l'aplicació
  s'executa sense desar fins a l'inici següent, i el registre ho diu.

## Llistes i sessió d'una versió anterior {#playlists-and-session-from-an-earlier-version}

Les versions anteriors desaven les llistes a `playlists.json`, les pàgines de
cartutxos a `carts.json` i la sessió a `session.json`. El primer inici
d'aquesta versió els llegeix (amb les seves còpies `.bak`) una vegada i ho
posa tot a `fauste.db`: la biblioteca amb els seus marcadors, etiquetes i
sonoritat, les llistes amb les seves marques de reproduïda, les pàgines de
cartutxos i la sessió. Després els fitxers vells es reanomenen amb `.imported`
al final (`playlists.json.imported`, `playlists.json.bak1.imported`, i així
successivament). Ja no s'usen i els pots esborrar quan estiguis segur que tot
és allà.

Per tornar a una versió anterior:

1. Tanca l'aplicació.
2. Mou `fauste.db`, `fauste.db-wal`, `fauste.db-shm` i tots els
   `fauste.db.bak*` a una carpeta fora de la carpeta de dades. No els esborris.
3. Treu `.imported` dels noms dels fitxers vells.
4. Inicia la versió més antiga.

Tot el que s'ha desat des de l'actualització es queda només a la carpeta on has
mogut els fitxers: els fitxers vells no ho tenen. Si actualitzes de nou més
endavant, els fitxers vells s'importen de nou, sempre que no hi hagi cap
`fauste.db` ni `fauste.db.bak*` a la carpeta de dades.

Si un inici s'interromp durant la importació, el següent l'acaba. Si un dels
fitxers vells no es pot llegir (massa gran, o escrit per una versió més nova),
l'aplicació no importa res i s'executa sense desar aquella sessió, de manera
que els fitxers vells es queden on són; el registre anomena el fitxer.
Arregla'l o treu-lo i torna a començar. Un cop existeix `fauste.db`, els
fitxers JSON vells no s'importen mai més.

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
