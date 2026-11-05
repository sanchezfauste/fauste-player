# Datos y copias de seguridad

## Dónde se guardan los archivos {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Configuración (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Biblioteca, playlists, cartuchos y sesión (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Caché de análisis | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Registros e informes de fallos | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Las rutas de Linux siguen las variables XDG (`XDG_CONFIG_HOME`, etc.) cuando
están definidas.

El snap guarda sus archivos en su propia carpeta: la configuración en
`~/snap/fauste-player/current/.config/fausteplayer/`, la base de datos y los
registros en `~/snap/fauste-player/current/.local/share/fausteplayer/`, y la
caché de análisis en `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` también los borra (conserva una instantánea
durante un tiempo; `--purge` no).

## Modo portátil {#portable-mode}

Si defines la variable de entorno `FAUSTE_HOME` con una carpeta, todo se
guarda allí, en `config/`, `data/`, `cache/` y `logs/`. Es útil en una
memoria USB, o para tener configuraciones separadas una al lado de otra.

## Archivos {#files}

| Archivo | Contenido |
|---|---|
| `config.json` | Configuración, salidas, umbrales de análisis, ajustes avanzados |
| `fauste.db` | La biblioteca (pistas, sus categorías, marcadores manuales, etiquetas, sonoridad), las playlists con sus marcas de reproducida, las páginas de cartuchos y la sesión: para cada player la playlist que se ve, las pistas actual y siguiente, el modo, la posición, el volumen y los anchos de columna (por columna), y la distribución de la ventana |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Copias de la base de datos (véase más abajo) |
| `window.ron` | El tamaño y la posición de la ventana principal y si estaba maximizada (solo en el ordenador; en Wayland el sistema elige la posición) |

## Guardado automático y copias de seguridad {#autosave-and-backups}

- Los cambios se guardan alrededor de un segundo después de producirse, y
  solo se escribe lo que cambió. Mientras algo suena, la sesión (las
  posiciones) se actualiza al mismo ritmo.
- La base de datos se escribe en transacciones, así que un corte de luz
  nunca deja un cambio a medio guardar.
- En el primer guardado de cada sesión, y luego más o menos cada cinco
  minutos mientras se guardan cambios, la aplicación copia la base de datos
  a `fauste.db.bak1` y desplaza las copias más antiguas a `.bak2` y `.bak3`.
  Por eso reiniciar varias veces seguidas empuja fuera las copias más
  antiguas.
- Si la base de datos no se puede leer, se usa la copia de seguridad buena
  más reciente. El archivo ilegible se conserva, renombrado como
  `fauste.db.corrupt-<time>`, para poder examinarlo. Si fallan todas las
  copias, la aplicación arranca vacía. Siempre arranca.
- Una copia que no se puede usar también se conserva: cuando la aplicación
  arranca desde otra copia (o desde una base de datos vacía), las inutilizables
  se renombran `fauste.db.bak1.corrupt-<time>` (o `.newer-<time>`) para que la
  siguiente copia de seguridad nunca las empuje fuera.
- Si otra copia de la aplicación está usando la base de datos, esta se
  ejecuta sin guardar y lo dice al arrancar.
- Una base de datos escrita por una versión más nueva de la aplicación no se
  abre: se conserva como `fauste.db.newer-<time>`, y la aplicación arranca
  desde la copia más reciente que pueda leer, o vacía si no hay ninguna, de
  modo que abrir los datos con una versión más antigua nunca los destruye.
- `config.json` se guarda igual que antes: un archivo temporal sustituye al
  anterior, y las versiones anteriores se conservan como `.bak1`, `.bak2` y
  `.bak3` a su lado.
- Si la base de datos no se puede usar en absoluto durante una sesión (por
  ejemplo, no se puede renombrar porque la carpeta es de solo lectura), la
  aplicación se ejecuta sin guardar hasta el siguiente arranque, y el
  registro lo dice.

## Playlists y sesión de una versión anterior {#playlists-and-session-from-an-earlier-version}

Las versiones anteriores guardaban las playlists en `playlists.json`, las
páginas de cartuchos en `carts.json` y la sesión en `session.json`. El
primer arranque de esta versión los lee (con sus copias `.bak`) una sola vez
y lo pone todo en `fauste.db`: la biblioteca con sus marcadores, etiquetas y
sonoridad, las playlists con sus marcas de reproducida, las páginas de
cartuchos y la sesión. Después los archivos antiguos se renombran con
`.imported` al final (`playlists.json.imported`,
`playlists.json.bak1.imported`, etcétera). Ya no se usan y puedes borrarlos
cuando estés seguro de que todo está ahí.

Para volver a una versión anterior:

1. Cierra la aplicación.
2. Mueve `fauste.db`, `fauste.db-wal`, `fauste.db-shm` y todos los
   `fauste.db.bak*` a una carpeta fuera de la carpeta de datos. No los
   borres.
3. Quita `.imported` de los nombres de los archivos antiguos.
4. Inicia la versión anterior.

Todo lo guardado desde la actualización se queda solo en la carpeta a la que
moviste los archivos: los archivos antiguos no lo tienen. Si actualizas otra
vez más adelante, los archivos antiguos se importan de nuevo, siempre que no
haya ningún `fauste.db` ni `fauste.db.bak*` en la carpeta de datos.

Si un arranque se interrumpe durante la importación, el siguiente la
termina. Si uno de los archivos antiguos no se puede leer (demasiado grande,
o escrito por una versión más nueva), la aplicación no importa nada y se
ejecuta sin guardar esa sesión, de modo que los archivos antiguos se quedan
donde están; el registro nombra el archivo. Arréglalo o quítalo y arranca de
nuevo. Una vez que existe `fauste.db`, los antiguos archivos JSON no se
importan nunca más.

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
