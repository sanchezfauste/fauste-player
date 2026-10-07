# Datos e copias de seguranza

## Onde se gardan os ficheiros {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Configuración (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Listas e sesión | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Caché de análise | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Rexistros e informes de fallos | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

As rutas de Linux seguen as variables XDG (`XDG_CONFIG_HOME` e así
sucesivamente) cando están definidas.

## Modo portátil {#portable-mode}

Define a variable de contorna `FAUSTE_HOME` cun cartafol, e todo se garda
alí, en `config/`, `data/`, `cache/` e `logs/`. Isto é útil nunha memoria
USB, ou para manter configuracións separadas ao carón.

## Ficheiros {#files}

| Ficheiro | Contido |
|---|---|
| `config.json` | Configuración, saídas, limiares de análise, axustes avanzados |
| `playlists.json` | Listas, pistas, marcas de reproducida, marcadores manuais |
| `session.json` | Para cada reprodutor: lista que mostra, pistas actual e seguinte, modo, posición, volume, anchos das columnas (por columna) |

## Gardado automático e copias de seguranza {#autosave-and-backups}

- Os cambios gárdanse aproximadamente un segundo despois de producirse.
  Mentres algo se reproduce, a sesión (as posicións) actualízase ao mesmo
  ritmo.
- Cada gardado escribe un ficheiro temporal e despois substitúe o antigo, así
  que un corte de corrente nunca deixa un ficheiro a medio escribir.
- As versións anteriores consérvanse como `.bak1`, `.bak2` e `.bak3`.
- Se un ficheiro non se pode ler, úsase a copia de seguranza boa máis recente.
  O ficheiro ilexible consérvase, renomeado `*.corrupt-<time>`, para
  inspeccionalo. A aplicación sempre arranca.

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
