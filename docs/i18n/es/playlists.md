# Playlists

## Pestañas {#tabs}

Cada player tiene una fila de pestañas, una por playlist. Todos los players
ven las mismas playlists; cada player elige cuál muestra. Un punto en una
pestaña indica dónde están las pistas del player: **rojo** para la pista que
suena y **verde** para la siguiente.

Las pestañas se reparten el ancho del player. Un nombre que no cabe termina
en «…»; pasa el puntero por encima de la pestaña para leerlo entero. Con
muchas playlists, las pestañas dejan de encogerse al llegar a un ancho
mínimo, aparecen flechas en los extremos de la fila y la rueda del ratón
sobre las pestañas las desplaza. La pestaña que eliges, y la que muestra un
player, se hace visible.

Cambiar de pestaña nunca cambia lo que suena ni lo que va a continuación.
Cuando acaba una pista, el player sigue en la playlist que contiene esa
pista.

Las playlists se crean, renombran y eliminan en
[Configuración](settings.md). No se pueden eliminar la última playlist ni
una playlist con una pista sonando.

## La tabla de pistas {#the-track-table}

![Una playlist: las pistas reproducidas atenuadas, la que suena en rojo, la siguiente en verde y el pie con el tiempo restante](../../images/guide/playlist.png)

| Columna | Contenido |
|---|---|
| `#` | Posición, con ceros a la izquierda; un icono la sustituye en la pista actual y en la siguiente |
| Título | De las etiquetas, o del nombre del archivo (`Artista - Título.mp3` se separa). Los iconos de repetir y de parar después de una pista van delante |
| Artista | De las etiquetas; «Artista desconocido» si no hay |
| Álbum | De las etiquetas |
| Fecha | La fecha de grabación tal como la guarda el archivo (`2019`, `2019-05` o `2019-05-14`, con hora si la tiene) |
| Género | De las etiquetas |
| Dur. | Duración de reproducción, del cue-in al cue-out (el archivo entero con **Usar cue-in y cue-out** desactivado) |
| Intro | Cuánto dura la intro, desde donde empieza a sonar la pista hasta su marcador de intro; vacía si la pista no tiene marcador de intro |
| Archivo | El nombre del archivo, con su extensión |

Una instalación nueva muestra `#`, Título, Artista y Dur. Las demás columnas
son opcionales; consulta **Elegir las columnas** más abajo. Una pista a la
que le falta un valor muestra una celda vacía, salvo en Artista, que muestra
«Artista desconocido».

Las columnas llenan la tabla y mantienen sus proporciones al cambiar el
tamaño de la ventana; las columnas de texto reciben más espacio. Arrastra
los separadores de la cabecera para cambiar las proporciones: las columnas a
la derecha del separador siguen al puntero en cada fotograma (se reparten lo
que queda en proporción a su ancho), las de la izquierda se quedan como
están, y los anchos se guardan al soltar. Ninguna columna se hace más
estrecha que su mínimo. Los anchos se recuerdan por player; una columna que
muestres más tarde empieza con su ancho por defecto y las demás mantienen
sus proporciones.

### Elegir las columnas {#choosing-the-columns}

Título y Dur. siempre se muestran. Cualquier otra columna se puede mostrar u
ocultar, y cualquier columna, incluidas esas dos, se puede mover. La lista es
la misma para todos los players y playlists, y se guarda en `config.json`
como `ui.table_columns`. Hay tres formas de cambiarla:

- **Configuración → Playlists → Columnas de la tabla:** marca las columnas
  que quieres ver; las flechas suben o bajan una columna visible (se leen de
  izquierda a derecha en las tablas). **Columnas por defecto** vuelve a `#`,
  Título, Artista y Dur.
- **Clic derecho en una cabecera:** un menú con una casilla para cada
  columna opcional. Una columna que muestres aparece en el extremo derecho;
  arrástrala desde allí.
- **Arrastra una cabecera** sobre otra: suéltala en la mitad izquierda de
  una cabecera para poner la columna delante, o en la mitad derecha para
  ponerla detrás. Soltarla en cualquier otro sitio no hace nada.

Un nombre de `ui.table_columns` que esta versión no conoce se ignora, y si
faltan Título o Dur. se vuelven a añadir.

Al abrir la aplicación, cada tabla se desplaza para que la siguiente pista
de su player quede en el centro de la tabla (tan cerca como permitan los
extremos de la lista). Esto ocurre una vez, al arrancar, y solo si la
siguiente pista está en la playlist que muestra la tabla.

Cuando un player pasa a otra pista, su tabla muestra la playlist de esa
pista y lleva su fila arriba del todo, salvo que hayas usado la tabla en los
últimos 10 segundos (la hayas desplazado, hayas arrastrado una pista,
abierto el menú de una pista o hecho clic en una pestaña): entonces espera
hasta que la dejes tranquila ese tiempo. El tiempo es
`ui.follow_current_grace_secs` en `config.json`; 0 desactiva el seguimiento.

Los players son independientes: varios players pueden mostrar la misma
playlist, cada uno con su propia siguiente pista, sus propias marcas de
reproducida y sus propios tiempos en el pie. Reproducir, detener o saltar en
un player nunca mueve la siguiente de otro player. La misma pista puede
incluso sonar en dos players a la vez. Aun así, editar la playlist (añadir,
mover o quitar entradas) o que un archivo deje de poder leerse puede cambiar
la siguiente de cualquier player que la muestre.

Colores de las filas:

| Fila | Significado |
|---|---|
| **Roja**, con un icono de altavoz (o de pausa) | Sonando en este player. También puede mostrar la flecha verde: la pista que suena es también la siguiente, así que se reproduce una vez más |
| **P2** (u otro número) en rojo en la columna del número | Sonando en ese player |
| **Verde**, con una flecha | La siguiente pista de este player |
| Atenuada | Ya reproducida en este player |
| Archivo con una cruz / icono de aviso | Archivo que falta / ilegible (se salta); pasa el puntero por la fila: la ventana emergente empieza con el motivo, en ámbar, y después los campos habituales. Un archivo que falta se vuelve a buscar cada 30 s (`tuning.missing_recheck_ms`). |
| Flechas de recarga a la derecha del título | Analizada por una versión anterior; sigue sonando con ese análisis. **Configuración → Análisis → Analizar las pistas desactualizadas** la pone al día (las pistas cargadas en un player se actualizan igualmente) |
| Reloj de arena a la derecha del título | La pista espera su análisis (pasa el puntero por el reloj de arena: *Análisis pendiente*). Suena igualmente, y el reloj de arena desaparece cuando termina el análisis |
| Violeta | Seleccionada |

**Información emergente de la pista.** Deja el puntero un momento sobre una
fila para ver su título, artista, álbum, fecha, género, duración, formato
(tipo, frecuencia de muestreo y profundidad de bits cuando se conocen) y la
ruta de su archivo. Un campo que el archivo no tiene se omite. La ventana
emergente es la única información al pasar el puntero por una fila, y nunca
se mueve mientras se muestra. Para un archivo que falta o es ilegible,
empieza con el motivo.

## Ratón {#mouse}

- **Clic** selecciona una pista. **Doble clic** la convierte en la siguiente
  pista de este player. Sobre la pista que suena, hace que se reproduzca una
  vez más cuando acabe la pasada en curso.
- **Clic derecho** abre el menú contextual:

![El menú contextual de una pista](../../images/guide/track-menu.png)

| Elemento | Acción |
|---|---|
| Reproducir ahora | Inicia esta pista al momento (con mezcla si el player está sonando) |
| Marcar como siguiente | Igual que el doble clic. Sobre la pista que suena, se reproduce una vez más, desde su inicio, cuando acaba la pasada en curso (con mezcla como Repetir, sin hueco), y después el player sigue. Actúa una sola vez. Stop al final, el modo SINGLE y una marca de Parar después de esta pista siguen deteniendo el player antes. Si hay un CUE en marcha, pasa a la nueva siguiente |
| Preescuchar en CUE | La reproduce en la salida CUE (abre la ventana de CUE). Aparece atenuado si el player no tiene una salida Cue distinta de su salida Main |
| Editar etiquetas… | Abre el editor de etiquetas de esta pista. **Guardar** escribe los cambios en el archivo de audio; **Cancelar** (o Esc, si no hay un guardado en curso) cierra sin escribir. El elemento aparece atenuado, con el motivo al pasar el puntero, mientras la pista suena, está en CUE o en un cartucho que suena, mientras aún no se han leído sus etiquetas, cuando el archivo falta, y en los formatos cuyas etiquetas no se pueden escribir (por ejemplo DSD) |
| Volver a analizar | Analiza esta pista otra vez ahora, sea cual sea su estado. Un archivo ilegible que se haya arreglado también se recoge por sí solo (consulta [Solución de problemas](troubleshooting.md)). Los marcadores manuales se conservan |
| Añadir pistas debajo… | Elige archivos para insertarlos después de esta pista |
| Duplicar | Inserta debajo una copia sin reproducir (con sus marcas de repetir y de parar después) |
| Repetir esta pista | Márcalo para reproducirla una y otra vez, sin hueco, hasta que pulses Play (siguiente), Anterior, Stop o Stop con fundido, o actives Stop al final. La pausa mantiene la repetición. Aparece un icono de repetir delante del título |
| Parar después de esta pista | Márcalo para detener el player cuando acabe esta pista, cada vez que suene (en cualquier modo). A diferencia del botón **Stop al final** del player, la marca va con la pista y se guarda con la playlist. El icono de parar después aparece delante del título. Tiene prioridad sobre Repetir |
| Mover a ▸ | La mueve al final de otra playlist |
| Quitar de la playlist | La quita; no es posible mientras suena |

## Editar etiquetas {#editing-tags}

**Editar etiquetas…** abre una ventana para una pista. Mientras está abierta
no actúa ningún atajo de teclado, y se ignoran los archivos que se sueltan
en la ventana de la aplicación.

![El editor de etiquetas de un archivo FLAC, con su carátula, título, artista, álbum, fecha y género](../../images/guide/tag-editor.png)

- **Qué ves.** El editor lee el archivo al abrirse (mientras tanto muestra
  «Leyendo etiquetas…»). Siempre se muestran: título, artista, álbum,
  artista del álbum, fecha, número de pista y total, número de disco y
  total, género, compositor y comentario. Se muestran cuando el archivo los
  tiene: subtítulo, agrupación, BPM, tonalidad inicial, estado de ánimo,
  ISRC, editorial, número de catálogo, copyright, artista original, álbum
  original, fecha de lanzamiento original, letrista, director, remezclador,
  arreglista, intérprete, idioma, codificado por, letra, ordenar por título,
  ordenar por artista, ordenar por álbum, ordenar por artista del álbum,
  ordenar por compositor y web del artista.
- **Añadir campo.** El menú de debajo de los campos lista los demás campos.
  Solo ofrece lo que el formato de etiquetas del archivo puede guardar (un
  WAV con RIFF INFO, un AIFF o una etiqueta ID3v1 antigua guardan menos
  campos que ID3v2, FLAC o MP4), y aparece atenuado cuando no queda nada que
  añadir. Uno de los campos que siempre se muestran y que el formato no
  puede guardar aparece en gris con una nota. Vaciar un campo lo quita del
  archivo; un campo añadido que se deja vacío no se escribe.
- **Varios valores.** Los campos que pueden tener varios valores (artista,
  artista del álbum, género, compositor, estado de ánimo y los créditos como
  letrista, director, remezclador, arreglista e intérprete, e idioma)
  muestran un valor por línea; **Guardar** escribe un valor por línea a la
  manera propia del formato. Comentario y letra son texto libre en varias
  líneas.
- **Comprobaciones.** La fecha y la fecha de lanzamiento original siguen
  ISO 8601 (`2019`, `2019-05` o `2019-05-14`, opcionalmente con hora); los
  números de pista y de disco, sus totales y el BPM son números enteros, y
  un total necesita su número. Un campo con un valor no válido queda marcado
  y **Guardar** sigue desactivado. Un valor que el archivo ya tenía y que no
  has tocado se conserva tal cual.
- **Campos demasiado largos.** Un campo cuyo texto supera
  `limits.max_tag_chars`, o que tiene más valores que
  `limits.max_tag_values`, se muestra en solo lectura con la nota
  «Demasiado largo para editarlo aquí; se conserva tal cual en el archivo».
  Nunca se vuelve a escribir, así que un guardado no puede recortarlo.
- **Qué se conserva.** Todo lo que el editor no muestra (otras claves
  estándar, claves personalizadas, imágenes distintas de la carátula
  frontal, tramas binarias) se queda en el archivo con los mismos valores.
  El editor dice cuántas de esas etiquetas se conservan (y «otras que no se
  cuentan» cuando el formato tiene tramas que no se pueden contar). Al
  guardar se vuelven a codificar los elementos que el editor gestiona, así
  que un elemento conservado puede cambiar en sus bytes (codificación del
  texto, orden de las tramas) pero no en su valor.
- **La carátula.** El editor muestra como miniatura la carátula frontal, o
  la primera imagen del archivo si no hay carátula frontal.
  - **Cambiar…** abre un diálogo de archivos para elegir una imagen JPEG o
    PNG (como mucho `limits.max_cover_bytes`, y debe poder decodificarse).
    Si no es así, el editor explica por qué y no cambia nada.
  - **Quitar** borra la carátula frontal. Está desactivado si el archivo no
    tiene carátula frontal: una imagen que se muestra solo porque no hay
    carátula frontal es únicamente para verla y se conserva tal cual.
  - Una carátula que está en el archivo pero no se puede mostrar (una imagen
    que no se decodifica, o un GIF, BMP o WebP) se indica con «Esta carátula
    no se puede mostrar; se conserva tal cual». **Cambiar…** y **Quitar**
    siguen funcionando.
  - El cambio lo escribe **Guardar** y lo descarta **Cancelar**. Las
    contraportadas y cualquier otra imagen nunca se tocan. Un formato sin
    sitio para imágenes (WAV con RIFF INFO, AIFF, ID3v1) muestra la zona
    desactivada. Tras guardar, la carátula del player muestra la nueva.
- **Cómo funciona un guardado.** El archivo se copia junto al original, la
  copia recibe las etiquetas, se sincroniza con el disco y sustituye al
  original, así que un fallo deja el archivo como estaba. El motivo aparece
  en el editor, que se queda abierto para volver a intentarlo, y en la barra
  de estado. Solo se escriben los campos que has cambiado. Tras guardar, la
  tabla muestra al momento las etiquetas nuevas, y los marcadores y la forma
  de onda se conservan. Si el archivo no conservó un campo que cambiaste, la
  barra de estado lo nombra.
- **Tras una actualización.** A las pistas de una versión anterior se les
  rellenan discretamente en segundo plano la fecha, el género y otras
  etiquetas (sin un análisis completo).

## Arrastrar y soltar {#drag-and-drop}

- Arrastra una pista dentro de la lista para reordenarla. Una línea violeta
  muestra dónde quedará: está en el borde entre filas más cercano al puntero,
  y solo en la lista que hay bajo el puntero. Soltarla sobre la cabecera, el
  borde de una columna, la barra de desplazamiento o una ventana que tape la
  lista (la ventana de CUE) no suelta nada.
- Mientras arrastras una pista, mantén el puntero cerca del borde superior o
  inferior de una lista para desplazarla: cuanto más cerca del borde, más
  rápido va, y se detiene en los extremos de la lista o cuando te alejas del
  borde. La rueda del ratón también desplaza la lista durante el arrastre.
  La línea violeta sigue al puntero mientras la lista se mueve. Arrastrar
  archivos desde el gestor de archivos sobre una lista la desplaza igual
  allí donde el sistema informa de la posición del puntero, en cuanto mueves
  el puntero sobre la lista.
- Arrástrala a la lista de otro player para moverla allí.
- Arrástrala a una pestaña para añadirla al final de esa playlist.
- Suelta archivos o carpetas desde el gestor de archivos sobre una lista
  para insertarlos en la posición donde los sueltas. Sobre la cabecera, el
  borde de una columna, la barra de desplazamiento o una ventana que tape la
  lista no se inserta nada. Si el sistema no informa de la posición, van al
  final de la lista que se ve.

## Pie {#footer}

**+ Añadir** abre un diálogo de archivos que empieza en la carpeta de música
fijada en Configuración. **Reiniciar** (el icono de la flecha que tiene al
lado) quita la marca atenuada de «ya reproducida» de todas las pistas de la
playlist, para todos los players, después de preguntar «¿Quitar la marca de
reproducida a todas las pistas de esta playlist?» (**Cancelar**, Esc o un
clic fuera conservan las marcas). La pista que suena conserva su estado y se
marca cuando el player la deja. El botón aparece atenuado cuando no hay nada
que quitar. El pie también muestra el número de pistas, el tiempo que queda
en la playlist y su duración total.

## Archivos de playlist {#playlist-files}

- **Importar:** Configuración → Playlists → **Importar M3U / PLS…**, o
  suelta un archivo `.m3u`, `.m3u8` o `.pls` en la ventana. Se convierte en
  una playlist nueva con el nombre del archivo.
  - Las rutas relativas se resuelven respecto a la carpeta del archivo de
    playlist.
  - Se entienden las direcciones `file://`.
  - Los archivos que no se encuentran se añaden igualmente, marcados como no
    disponibles.
  - Los streams de Internet se omiten; un mensaje dice cuántos.
- **Exportar:** el botón **M3U** de cada playlist en Configuración la guarda
  como un archivo M3U8 con títulos, duraciones y rutas completas.
