# Marcadores y mezcla

Cada pista tiene hasta cinco **marcadores**, en segundos:

| Marcador | Significado | Cómo se fija |
|---|---|---|
| Cue-in | Donde empieza la reproducción | Automático: justo antes del primer sonido por encima del umbral de recorte |
| Cue-out | Donde acaba la pista | Automático: justo después del último sonido por encima del umbral de recorte |
| MIX (inicio del segue) | Donde empieza la siguiente pista en modo continuo | Automático (ver más abajo) |
| Inicio del outro | Donde empieza el final de la pista | Automático (ver más abajo) |
| Fin de la intro | Fin de la introducción sobre la que se puede hablar | A mano, o desde una etiqueta `INTRO` del archivo |

Los marcadores fijados a mano siempre tienen prioridad: un análisis nuevo
nunca los sustituye.

## Editar marcadores {#editing-markers}

En la forma de onda de un player, o en la de su ventana de CUE (el mismo
menú y los mismos tiradores; un cambio se ve en las dos a la vez):

- **Haz clic derecho** donde quieras un marcador y elige **Poner el inicio
  aquí**, **Poner el fin de la intro aquí**, **Poner el inicio del outro
  aquí**, **Poner el punto MIX aquí** o **Poner el final aquí**. **Volver a
  los marcadores automáticos** quita los marcadores que hayas colocado, y la
  pista se vuelve a analizar.
- **Mantén pulsado Alt** (Opción en macOS): aparecen tiradores en los
  marcadores. Arrastra uno para moverlo; el tiempo se muestra mientras
  arrastras. Arrastrar nunca mueve el cabezal.

El cue-in debe quedar antes del cue-out. Los demás marcadores se mantienen
entre los dos. Los cambios en la pista que suena se aplican al momento a su
siguiente transición.

## La etiqueta INTRO {#the-intro-tag}

Un archivo puede llevar el tiempo de su intro en una etiqueta `INTRO`, en
segundos (`12.5`) o como `m:ss`. El nombre puede escribirse en mayúsculas o
minúsculas (`INTRO`, `Intro`). Puede ser una trama de texto de usuario ID3v2
(MP3, WAV, AIFF, DSF), un comentario Vorbis, Opus o FLAC, un elemento APE
(WavPack, Monkey's Audio) o un átomo libre de MP4. Se lee durante el
análisis. Un fin de intro manual sigue teniendo prioridad.

## Cómo se encuentran los marcadores automáticos {#how-the-automatic-markers-are-found}

El análisis mide los picos de la pista en pasos de 10 ms y su sonoridad en
ventanas cortas (50 ms por defecto).

- **Cue-in / cue-out:** solo se salta el casi silencio del principio y del
  final: todo lo que tenga un pico que llegue al *umbral de recorte*
  (−60 dBFS por defecto), en cualquiera de los canales, se conserva, con un
  *margen de recorte* (20 ms por defecto) alrededor. Las entradas suaves, las
  colas tranquilas y los sonidos cortos nunca se cortan.
- **MIX:** el análisis encuentra el último punto en el que la pista sigue
  estando menos de la *caída para el segue* (15 dB por defecto) por debajo
  de su sonoridad habitual, así que masters fuertes y suaves con el mismo
  fundido se mezclan igual. Ese punto nunca está más de la *duración máxima
  de la mezcla* (4 s por defecto) antes del cue-out, para que los solapes
  sean cortos.
- **Outro:** el análisis recorre la pista hacia atrás desde el cue-out y
  encuentra dónde el nivel cae más de la *caída de nivel del outro* (6 dB
  por defecto) por debajo de la sonoridad mediana de la pista. Por defecto,
  el outro nunca dura más de 30 s.
- Las pistas más cortas que la *duración mínima para marcadores de mezcla y
  outro* (60 s por defecto), como jingles y anuncios, no reciben ni MIX ni
  outro.

### Grabaciones largas {#long-recordings}

Un programa entero (de una, cuatro o más horas) se analiza como una canción,
mientras suena si hace falta: un archivo FLAC u Opus de 4 horas tarda menos
de un minuto en un ordenador actual, y la memoria no crece con la duración.
Saltar a cualquier punto, incluso cerca del final, es inmediato. La forma de
onda y los marcadores se guardan en la caché de análisis hasta unas 16 horas
de audio; un archivo más largo también funciona, pero se vuelve a analizar
cada vez que arranca Fauste Player.

Todos estos valores están en **Configuración → Análisis**. Después de
cambiarlos, las pistas se vuelven a analizar automáticamente.

## Qué hace el player con ellos {#what-the-player-does-with-them}

- **Modo continuo con la mezcla automática activada:** en el punto MIX la
  siguiente pista empieza a pleno nivel mientras la actual baja con un
  fundido hasta su cue-out. El solape es exacto a nivel de muestra.
- **Modo continuo sin punto MIX,** o con la mezcla automática desactivada: la
  siguiente pista empieza exactamente en el cue-out, sin hueco.
- **Modo single**, o **Stop al final**: el player se detiene en el cue-out.
- **Pulsar Play mientras suena:** la siguiente pista empieza al momento y la
  actual baja con un fundido durante la *duración del fundido* (1 s por
  defecto).
- **Usar cue-in y cue-out desactivado** (Configuración → Players): todos los
  players reproducen cada pista desde 0 hasta el final del archivo. El cue-in
  y el cue-out, automáticos y manuales, se conservan, y la forma de onda los
  dibuja como líneas tenues. El punto MIX, la intro y el outro siguen
  funcionando, dentro del archivo entero; **Mezcla automática en el punto
  MIX** es un interruptor aparte. Las cuentas atrás, la columna de duración,
  los totales de las playlists en Configuración y los tiempos de la API
  remota siguen el mismo rango. Los cartuchos siempre usan su propio cue-in
  y cue-out. Cambiar el ajuste nunca reinicia, mueve ni detiene una pista
  que está sonando; la siguiente pista se vuelve a preparar.

Una pista se puede reproducir antes de que termine su análisis. Hasta
entonces suena desde el principio hasta el final del archivo, sin punto MIX.
