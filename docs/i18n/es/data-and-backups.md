# Datos y copias de seguridad

## Dónde se guardan los archivos {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Configuración (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Playlists y sesión | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Caché de análisis | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Registros e informes de fallos | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Las rutas de Linux siguen las variables XDG (`XDG_CONFIG_HOME`, etc.) cuando
están definidas.

## Modo portátil {#portable-mode}

Si defines la variable de entorno `FAUSTE_HOME` con una carpeta, todo se
guarda allí, en `config/`, `data/`, `cache/` y `logs/`. Es útil en una
memoria USB, o para tener configuraciones separadas una al lado de otra.

## Archivos {#files}

| Archivo | Contenido |
|---|---|
| `config.json` | Configuración, salidas, umbrales de análisis, ajustes avanzados |
| `playlists.json` | Playlists, pistas, marcas de reproducida, marcadores manuales |
| `session.json` | Para cada player: playlist que se ve, pistas actual y siguiente, modo, posición, volumen, anchos de columna (por columna) |

## Guardado automático y copias de seguridad {#autosave-and-backups}

- Los cambios se guardan alrededor de un segundo después de producirse.
  Mientras algo suena, la sesión (las posiciones) se actualiza al mismo
  ritmo.
- Cada guardado escribe un archivo temporal y luego sustituye al anterior,
  así que un corte de luz nunca deja un archivo a medio escribir.
- Las versiones anteriores se conservan como `.bak1`, `.bak2` y `.bak3`.
- Si un archivo no se puede leer, se usa la copia de seguridad buena más
  reciente. El archivo ilegible se conserva, renombrado como
  `*.corrupt-<time>`, para poder examinarlo. La aplicación siempre arranca.

## Recuperación tras un fallo {#crash-recovery}

Después de un fallo o de un reinicio, cada player vuelve con su playlist,
sus pistas actual y siguiente y su posición, pero **en pausa o detenido**.
No sale nada al aire por sí solo. Un player que estaba justo al final de su
pista vuelve al principio de esa pista, para que pulsar Play la reproduzca
en lugar de terminarla al instante.

## Editar `config.json` a mano {#editing-configjson-by-hand}

Cierra antes la aplicación (si hay audio al aire, pregunta antes de
cerrarse). Los valores desconocidos o fuera de rango se corrigen al valor
válido más cercano al cargar el archivo, y las correcciones quedan en el
registro. Las secciones `limits` y `tuning` contienen valores avanzados
(límites de recursos, temporización del motor) que no están en la ventana de
Configuración.
