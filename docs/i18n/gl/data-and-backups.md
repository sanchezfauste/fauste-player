# Datos e copias de seguranza

## Onde se gardan os ficheiros {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Configuración (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Biblioteca, listas, cartuchos e sesión (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Caché de análise | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Rexistros e informes de fallos | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

As rutas de Linux seguen as variables XDG (`XDG_CONFIG_HOME` e así
sucesivamente) cando están definidas.

O snap garda os seus ficheiros no seu propio cartafol: a configuración en
`~/snap/fauste-player/current/.config/fausteplayer/`, a base de datos e os
rexistros en `~/snap/fauste-player/current/.local/share/fausteplayer/`, e a
caché de análise en `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` bórraos tamén (conserva unha instantánea
durante un tempo; `--purge` non).

## Modo portátil {#portable-mode}

Define a variable de contorna `FAUSTE_HOME` cun cartafol, e todo se garda
alí, en `config/`, `data/`, `cache/` e `logs/`. Isto é útil nunha memoria
USB, ou para manter configuracións separadas ao carón.

## Ficheiros {#files}

| Ficheiro | Contido |
|---|---|
| `config.json` | Configuración, saídas, limiares de análise, axustes avanzados |
| `fauste.db` | A biblioteca (pistas, as súas categorías, marcadores manuais, etiquetas, sonoridade), as listas coas súas marcas de reproducida, as páxinas de cartuchos e a sesión: para cada reprodutor a lista que mostra, as pistas actual e seguinte, o modo, a posición, o volume e os anchos das columnas (por columna), e a disposición da xanela |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Copias da base de datos (ver abaixo) |
| `window.ron` | O tamaño e a posición da xanela principal e se estaba maximizada (só no computador; en Wayland o sistema escolle a posición) |

## Gardado automático e copias de seguranza {#autosave-and-backups}

- Os cambios gárdanse aproximadamente un segundo despois de producirse, e só
  se escribe o que cambiou. Mentres algo se reproduce, a sesión (as
  posicións) actualízase ao mesmo ritmo.
- A base de datos escríbese en transaccións, así que un corte de corrente
  nunca deixa un cambio a medio gardar.
- No primeiro gardado de cada sesión, e despois cada cinco minutos
  aproximadamente mentres se gardan cambios, a aplicación copia a base de
  datos a `fauste.db.bak1` e despraza as copias máis antigas a `.bak2` e
  `.bak3`. Reiniciar varias veces seguidas, polo tanto, empurra fóra as
  copias máis antigas.
- Se a base de datos non se pode ler, úsase a copia de seguranza boa máis
  recente. O ficheiro ilexible consérvase, renomeado
  `fauste.db.corrupt-<time>`, para inspeccionalo. Se fallan todas as copias,
  a aplicación arranca baleira. Sempre arranca.
- Tamén se conserva unha copia que non se pode usar: cando a aplicación
  arranca desde outra copia (ou desde unha base de datos baleira), as que
  non serven renómeanse `fauste.db.bak1.corrupt-<time>` (ou
  `.newer-<time>`) para que a seguinte copia nunca as empurre fóra.
- Se outra copia da aplicación está a usar a base de datos, esta funciona
  sen gardar e dío ao arrancar.
- Unha base de datos escrita por unha versión máis nova da aplicación non se
  abre: consérvase como `fauste.db.newer-<time>`, e a aplicación arranca
  desde a copia máis recente que poida ler, ou baleira se non hai ningunha,
  así que abrir os datos cunha versión máis antiga nunca os destrúe.
- `config.json` gárdase como antes: un ficheiro temporal substitúe o antigo,
  e as versións anteriores consérvanse como `.bak1`, `.bak2` e `.bak3` ao seu
  carón.
- Se a base de datos non se pode usar en absoluto durante unha sesión (por
  exemplo, non se pode renomear porque o cartafol é de só lectura), a
  aplicación funciona sen gardar ata o seguinte arranque, e o rexistro dío.

## Listas e sesión dunha versión anterior {#playlists-and-session-from-an-earlier-version}

As versións anteriores gardaban as listas en `playlists.json`, as páxinas de
cartuchos en `carts.json` e a sesión en `session.json`. O primeiro arranque
desta versión léas (coas súas copias `.bak`) unha vez e pon todo en
`fauste.db`: a biblioteca cos seus marcadores, etiquetas e sonoridade, as
listas coas súas marcas de reproducida, as páxinas de cartuchos e a sesión.
Despois os ficheiros antigos renómeanse con `.imported` ao final
(`playlists.json.imported`, `playlists.json.bak1.imported` e así
sucesivamente). Xa non se usan e podes eliminalos cando esteas seguro de que
todo está aí.

Para volver a unha versión anterior:

1. Pecha a aplicación.
2. Move `fauste.db`, `fauste.db-wal`, `fauste.db-shm` e cada
   `fauste.db.bak*` a un cartafol fóra do cartafol de datos. Non os elimines.
3. Quita `.imported` dos nomes dos ficheiros antigos.
4. Inicia a versión anterior.

Todo o que se gardou desde a actualización queda só no cartafol ao que
moveches os ficheiros: os ficheiros antigos non o teñen. Se actualizas de
novo máis tarde, os ficheiros antigos impórtanse outra vez, sempre que non
haxa ningún `fauste.db` nin `fauste.db.bak*` no cartafol de datos.

Se un arranque se interrompe durante a importación, o seguinte remátaa. Se un
dos ficheiros antigos non se pode ler (demasiado grande, ou escrito por unha
versión máis nova), a aplicación non importa nada e funciona sen gardar esa
sesión, así que os ficheiros antigos quedan onde están; o rexistro nomea o
ficheiro. Arránxao ou quítao e inicia de novo. Unha vez que existe
`fauste.db`, os ficheiros JSON antigos non se importan nunca máis.

## Recuperación tras un fallo {#crash-recovery}

Despois dun fallo ou dun reinicio, cada reprodutor volve coa súa lista, as
súas pistas actual e seguinte e a súa posición, pero **en pausa ou detido**.
Nada sae en antena por si só. Un reprodutor que estaba ao final mesmo da súa
pista volve ao comezo desa pista, de modo que premer Play a reproduce en
lugar de rematala ao instante.

## Editar `config.json` a man {#editing-configjson-by-hand}

Pecha primeiro a aplicación (se hai audio en antena, pregunta antes de
pechar). Os valores descoñecidos ou fóra de rango corríxense ao valor válido
máis próximo cando se carga o ficheiro, e as correccións quedan no rexistro.
As seccións `limits` e `tuning` conteñen valores avanzados (límites de
recursos, tempos do motor) que non están na xanela de Configuración.
