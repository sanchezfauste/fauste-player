# Android

Fauste Player también funciona en teléfonos y tabletas Android (Android 10 o
posterior), sobre el mismo núcleo de emisión que el escritorio: los
players, las playlists, la cartuchera, los marcadores, la normalización de
la sonoridad y todas las reglas de reproducción funcionan igual.

## Instalación {#install}

Una versión que incluye la aplicación lleva
`fauste-player-<version>-android.apk` en su página de **Releases** (con su
`.sha256`). Ábrelo en el teléfono y permite que tu navegador o gestor de
archivos instale aplicaciones cuando Android lo pida. Para actualizar,
instala el APK más reciente de la misma manera: tus playlists y ajustes se
conservan. La aplicación todavía no está en Google Play.

La versión 0.13.0 y las anteriores se instalan como otra aplicación, así que
un APK más reciente no las actualiza: aparece como un segundo Fauste Player.
Desinstala la antigua cuando la nueva esté configurada. Sus playlists y
ajustes no se trasladan.

## Primer arranque {#first-start}

- **Acceso a la música.** Permite el acceso a la música (y a las
  notificaciones, en Android 13 y posteriores). Sin él, el explorador de
  música se queda vacío.
- **La emisión sigue.** Mientras un player o un cartucho suena, se mantiene
  una notificación y el audio continúa con la pantalla apagada o con la
  aplicación en segundo plano. Como en el escritorio, nada sale al aire por
  sí solo tras un arranque o un fallo.
- **Atrás y cierre.** Atrás cierra primero lo que está abierto (el
  explorador de música, Configuración, Acerca de, un player abierto, una
  página de la pestaña Biblioteca). Si no queda nada que cerrar, Atrás guarda
  la aplicación mientras suena algo (un player, un cartucho o una
  preescucha en CUE) y sigue reproduciendo; cuando no suena nada, guarda y
  cierra la aplicación. Deslizar la aplicación fuera en el selector de
  aplicaciones hace lo mismo: deslizar nunca detiene lo que suena, y la
  aplicación se cierra cuando se queda en silencio.
- **El sonido de otras aplicaciones.** Cuando otra aplicación toma el
  audio, Fauste Player sigue reproduciendo y lo dice en la barra de estado
  de abajo.

## La pantalla {#the-screen}

En un **teléfono** la barra superior tiene cuatro pestañas, **Directo**,
**Cartuchera**, **Playlists** y **Biblioteca**, mostradas como iconos (la
elegida también con su nombre cuando hay sitio), y luego el **candado** y
**⋮** (Más: Configuración y Acerca de). El reloj aparece cuando hay sitio
para él. Con el teléfono girado, la pantalla Playlists pone sus chips de
player (P1, P2…) en la barra superior, y la barra de estado de abajo solo
aparece cuando tiene algo que decir.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Directo en un teléfono: dos players como tarjetas, una entera y el comienzo de la siguiente, y la cartuchera debajo">
<img src="../../images/android/2-carts.png" width="180" alt="Cartuchera en un teléfono: la página General en vertical, dos cartuchos por fila">
<img src="../../images/android/3-playlists.png" width="180" alt="Playlists en un teléfono: los chips de P1 a P4, las pestañas de playlist, la lista de pistas y la tarjeta del player abajo">
<img src="../../images/android/4-music.png" width="180" alt="Música del dispositivo en un teléfono: la carpeta Demo con tres canciones marcadas y la barra para añadirlas a la playlist Morning de P1">
<img src="../../images/android/5-library.png" width="180" alt="La pestaña Biblioteca en un teléfono: las pistas de la categoría Music, cada una con una casilla para marcar y unos auriculares para preescuchar, y debajo la tarjeta de preescucha con la forma de onda de la pista y sus marcadores">
</p>

- *Directo:* cada player como una tarjeta, y la cartuchera. Cada tarjeta
  tiene, de arriba abajo, el nombre del player, su estado, Single/Cont y
  CUE; la carátula, el título y el artista y la siguiente pista; la forma de
  onda (un toque salta); luego el tiempo transcurrido y la duración de la
  pista con el tiempo restante en dígitos grandes sobre el transporte
  (Play/Siguiente, Anterior, Pausa, Stop, Reiniciar, Stop con fundido, Stop
  al final) con botones del tamaño de un dedo, y un vúmetro delgado con el
  fader de volumen (el nivel del fader se muestra debajo). El tiempo
  restante termina donde terminan los botones, más pequeño para una pista
  larga. En la mayoría de los teléfonos el vúmetro muestra su escala a su
  izquierda, y en uno más ancho (unos 406 dp o más) el fader también muestra
  sus niveles; en un teléfono estrecho (menos de unos 384 dp) no lo hace
  ninguno de los dos, y los botones se hacen un poco más pequeños para que
  quepan. Los botones funcionan mientras otro dedo sujeta el fader, y un
  toque cuenta aunque el dedo ruede un poco o se quede apoyado un momento. En
  vertical, las tarjetas van arriba, una entera y el comienzo de la
  siguiente (desplázate para ver más), y la cartuchera debajo. Con el
  teléfono girado, cada player es una fila a la izquierda (su nombre y
  estado, el título, el tiempo restante, Play/Siguiente, Pausa y Stop; toca
  la fila en otro sitio para abrir el player en Playlists) y la cartuchera
  está a la derecha. Toca **CARTUCHERA** en su cabecera para plegarla: solo
  queda la cabecera, abajo (las pestañas de página y *Parar todo* siguen
  funcionando), y los players ganan el espacio; tócala otra vez para
  desplegarla. La pestaña Cartuchera siempre muestra la cartuchera entera.
  Una página de cartuchos demasiado ancha para el espacio se muestra en
  menos columnas, en el mismo orden.
- *Cartuchera:* la página de cartuchos visible, llenando la pantalla. Desliza
  hacia los lados para mostrar la página siguiente o la anterior. En
  vertical, una página más ancha que alta se gira (8 × 2 se muestra como
  2 × 8), en el mismo orden.
- *Playlists:* los chips de arriba (P1, P2…) eligen el player; debajo están
  sus pestañas de playlist (+ crea una nueva), la lista de pistas y, abajo
  (a la derecha cuando el teléfono está de lado), la tarjeta del player con
  una forma de onda mayor para preparar una pista (se acorta cuando falta
  sitio, de modo que los botones siempre se vean). Una pulsación larga en
  una fila abre su menú, que también tiene **Subir**, **Bajar** y **Mover a
  otro player** (la pista va justo después de la siguiente pista de ese
  player, en la playlist que muestra; la pista en antena se queda). Arrastra
  una fila por su asa ≡ para moverla. El pie tiene **Añadir** y, tras **⋯**,
  **Reiniciar**. Una pulsación larga en la forma de onda pone un marcador en
  ese punto (cue-in, fin de la intro, inicio del outro, MIX, cue-out) o
  quita uno que pusiste; los marcadores se editan solo aquí, no en Directo.
- *Biblioteca:* la biblioteca de pistas y la música del dispositivo (véase
  [Biblioteca](#library)).

Atrás desde otra pantalla va a Directo; en Biblioteca primero vuelve a la
lista de categorías (en Música del dispositivo, primero sube una carpeta).

Cuando nada se mueve en la pantalla, la aplicación la dibuja una vez por
segundo (por el reloj) para ahorrar batería; un vúmetro que sigue cayendo
tras una pausa o un stop la mantiene dibujando hasta que el vúmetro baja del
todo.

**El candado** protege de toques accidentales durante un programa. Bloqueado
(el candado cerrado y encendido), los faders de volumen solo muestran el
nivel y las formas de onda ni saltan ni editan marcadores; se ven
atenuados. Un pellizco y un arrastre siguen ampliando y moviendo una forma
de onda, y Play, el transporte, los cartuchos y las playlists funcionan
siempre. El bloqueo se mantiene tras un reinicio y se guarda con la sesión,
así que al cargar otra sesión se toma el bloqueo de esa sesión; la API
remota y el MIDI no se bloquean.

Un arrastre que empieza en un fader de volumen lo mueve; uno que empieza en
cualquier otro sitio desplaza, de modo que desplazarse nunca cambia un nivel
en antena. Un toque en una forma de onda salta a ese punto, un pellizco con
dos dedos la amplía alrededor de tus dedos, un arrastre con un dedo mueve
una vista ampliada y **Vista completa** vuelve a mostrar la pista entera. En
una playlist un dedo desplaza; para mover una pista, usa su asa ≡ o el menú
de la fila.

En una **tableta** la distribución es la del escritorio (véase
[Distribución](layout.md)), con objetivos más fáciles de pulsar.

Una pulsación larga abre lo que abre un clic derecho en el escritorio: el
menú de la fila, los menús de las cabeceras, el menú de un cartucho
(**Editar…** elige el archivo de un cartucho vacío). En un teléfono,
Configuración llena la pantalla y lista sus secciones; toca una para abrirla,
con el nombre de cada ajuste sobre su control. La flecha o Atrás vuelve a
la lista, y Atrás de nuevo (o **Cerrar**) cierra Configuración. Editar un
cartucho abre directamente su sección.

Acerca de → **Licencias de terceros** abre los avisos de las bibliotecas con
las que está construida la aplicación en un navegador o visor de HTML.

## Biblioteca {#library}

En un **teléfono** la pestaña **Biblioteca** se abre en una lista de
categorías: **Todas las pistas**, tus categorías (cada subcategoría bajo su
categoría madre, con su color y cuántas pistas contiene), **Sin categoría** y
**Música del dispositivo**. La biblioteca es la misma que muestra la
[ventana de la biblioteca](library.md) del escritorio: todo archivo añadido
a una playlist, a un cartucho o a la biblioteca está en ella. Las categorías
se crean, renombran, cambian de color y eliminan en **Configuración →
Biblioteca**.

**Las pistas de una categoría.** Toca una categoría para abrir sus pistas
(con las de sus subcategorías). La flecha de su cabecera, o Atrás, vuelve a
las categorías. Otra pestaña y volver encuentra la lista como la dejaste.
Abrir una categoría desde las categorías empieza con una búsqueda vacía. La
búsqueda filtra la lista por título, artista, álbum y las demás etiquetas
que busca la ventana de la biblioteca; **Toda la biblioteca** busca en todas
las categorías. Cada fila muestra el título (o el nombre del archivo), el
artista, la duración y las marcas de la ventana para un archivo que falta,
no se puede leer o aún no se ha analizado.

Toca filas para marcarlas. Mientras hay algo marcado, abajo aparece la barra
de Música del dispositivo: cuántas, el player y la playlist a los que van
(tócala para elegir otra), **Añadir al final** y **Añadir tras la
siguiente**, como en Música del dispositivo. Las pistas van en el orden en
que las marcaste, y añadir borra las marcas. Volver a las categorías
también las borra.

**El menú de la fila.** Una pulsación larga en una fila abre su menú:
**Añadir a ▸** (una playlist de un player, al final), **Preescuchar**,
**Mover a la categoría ▸**, **Volver a analizar** y **Quitar de la
biblioteca…**, que pregunta como en el escritorio (véase [Quitar
pistas](library.md#removing-tracks)). Mientras hay filas marcadas, una
pulsación larga en otra fila la marca también y el menú actúa sobre todas
las filas marcadas, en el orden en que las marcaste; **Preescuchar** entonces
no se ofrece. Las marcas se quedan tras la acción del menú.

**Preescucha.** Los 🎧 a la derecha de una fila reproducen su pista desde su
cue-in en la salida de preescucha de la biblioteca; los 🎧 de otra fila cambian
a esa pista. Una franja delgada sobre la lista muestra el título, el tiempo
restante, **Pausar**/**Preescuchar** y **Detener** mientras suena (o está en
pausa), el título atenuado tras **Detener**, y una indicación antes de la
primera preescucha: siempre conserva su sitio, de modo que las filas no se
mueven cuando empieza o acaba una preescucha. Los 🎧 se atenúan cuando el
archivo falta o no se puede reproducir.

**La tarjeta de preescucha.** Toca la franja (fuera de sus botones) para
abrir la tarjeta bajo la lista, con la forma de onda de la pista como en la
tarjeta de un player en Playlists; la flecha a la derecha de la franja
indica si está abierta. Toca la franja otra vez, o pulsa Atrás, para
cerrarla; la preescucha continúa. La flecha de la cabecera de la lista
vuelve a las categorías enseguida y deja la tarjeta abierta para la
siguiente lista. En la tarjeta:

- un toque en la forma de onda salta a ese punto o, con la preescucha
  detenida, fija dónde empieza **Preescuchar**;
- una pulsación larga pone un marcador en ese punto (cue-in, fin de la
  intro, inicio del outro, MIX, cue-out) o quita uno que pusiste. Los
  marcadores son de la pista, así que el cambio se ve en todas las
  playlists y en el escritorio;
- un pellizco amplía, un arrastre mueve una vista ampliada y **Vista
  completa** vuelve a mostrar toda la pista;
- bajo la forma de onda están los tiempos, **Preescuchar**, **Pausar**,
  **Detener** y el medidor.

La lista queda encima de la tarjeta (a la izquierda con el teléfono de
lado), así que los 🎧 de otra fila siguen cambiando la tarjeta a esa pista.
El candado detiene los saltos y los cambios de marcadores de la forma de
onda, como en los players.
La preescucha
se detiene cuando su pista sale de la biblioteca y cuando sales de la
pestaña Biblioteca (a otra pestaña, o con Atrás); en las categorías o en
Música del dispositivo continúa, pero su franja solo se muestra en la lista
de pistas.

A diferencia del escritorio, Android (un teléfono o una tableta) reproduce
la preescucha de la biblioteca en la salida por defecto cuando no se ha
elegido ninguna salida para ella: en **Configuración → Salidas de audio →
Biblioteca** la opción vacía dice *Predeterminado del sistema*, y su
**Probar Cue** suena allí. Elige otra salida allí (unos auriculares con
cable, por ejemplo) para preescuchar lejos del aire. La salida por defecto
suele ser también la de los players y la de la cartuchera, así que la
preescucha puede oírse en antena: mientras algo suena en esa salida, los 🎧
preguntan primero (**Preescuchar igualmente** o **Cancelar**), y una
preescucha que no confirmaste se detiene cuando algo empieza allí, como el
CUE de un player (véase [CUE (preescucha)](players.md#cue-pre-listen)).

**Añadir música del dispositivo a una categoría.** El **＋** de la cabecera
de una lista de pistas abre Música del dispositivo para esa categoría. Su
barra tiene entonces solo **Añadir a &lt;categoría&gt;** (**Añadir a Sin
categoría** desde *Todas las pistas* o *Sin categoría*), que añade los
archivos marcados a la biblioteca en esa categoría y vuelve a la lista; un
archivo que ya está en la biblioteca entra en ella si no tiene categoría y
conserva la que tiene en caso contrario, como en el escritorio. Música del
dispositivo conserva sus marcas entre visitas, así que también se añaden
los archivos marcados en una visita anterior: comprueba primero el recuento
de la barra. **Volver a la lista**, arriba, o Atrás vuelve a la lista sin
añadir. Música del dispositivo abierta desde las categorías tiene arriba
**Volver a las categorías** en su lugar.

**La carpeta del dispositivo de una categoría.** Una categoría puede tomar
sus archivos de una carpeta de la música del dispositivo. En su menú (una
pulsación larga en ella en **Configuración → Biblioteca**, o en la ventana
de la biblioteca en una tableta), **Usar una carpeta del dispositivo…** abre
Música del dispositivo para elegir la carpeta (**Usar esta carpeta**); el
menú de la categoría la nombra entonces, sobre sus elementos de carpeta.
Nada vigila la carpeta: cuando hayas copiado música nueva allí, toca el
**↻** de la cabecera de la lista de pistas de la categoría (o **Releer
carpeta del dispositivo** en su menú). Todo archivo que Música del
dispositivo lista en la carpeta y en sus subcarpetas se añade a la
categoría, con las reglas anteriores (un archivo que ya está en otra
categoría conserva esa), y un aviso dice cuántos archivos se leyeron. Los
archivos que salieron de la carpeta se quedan en la biblioteca, marcados como faltantes, y vuelven a la normalidad cuando
regresan a ella (cuando Música del dispositivo no lista nada, por ejemplo sin
permiso, no se marca nada). **Cambiar
carpeta del dispositivo…** y **Olvidar carpeta del dispositivo** están en el
mismo menú; una carpeta pertenece a una sola categoría.

### Música del dispositivo {#device-music}

**Música del dispositivo** muestra los archivos de audio del dispositivo por
**Carpetas** (con la ruta sobre la lista; toca una parte para volver
arriba), **Artistas** o **Álbumes**, y una búsqueda por título, artista,
álbum y nombre de archivo. Cada fila muestra el título, el artista y la
duración; toca filas para marcarlas. Mientras hay algo marcado, una barra
abajo muestra cuántas, el player y la playlist a los que van (tócala para
elegir otra), **Añadir al final** y, cuando la siguiente pista de ese
player está en esa playlist, **Añadir tras la siguiente**. Las pistas van
en el orden en que las marcaste. Un archivo del dispositivo no tiene
preescucha.

**Añadir** en el pie de una playlist, el archivo de un cartucho y
Configuración → Playlists → Carpeta de música abren Música del dispositivo
en un teléfono (en una tableta, la ventana del **explorador de música**, por
carpetas). Marca archivos y toca **Añadir**, toca un archivo para un
cartucho, o **Usar esta carpeta**; **Cancelar** o Atrás vuelve a donde
estabas. Copia música al dispositivo (por ejemplo a su carpeta *Música*)
para verla allí. Fauste Player solo lee tu música: las etiquetas y las
carátulas no se escriben, y las playlists y las páginas de cartuchos todavía
no se importan ni se exportan en Android.

En una **tableta** la biblioteca es la [ventana de la
biblioteca](library.md) del escritorio, que se abre desde la barra
superior.

## Salidas {#outputs}

Configuración → Salidas de audio lista las salidas del teléfono por tipo: el
altavoz, auriculares con cable, una interfaz USB, un altavoz Bluetooth
(«JBL Flip (Bluetooth)»). Cada player, la cartuchera y sus CUE eligen una,
como en el escritorio, y también la preescucha de la biblioteca (véase
[Biblioteca](#library)). Un altavoz Bluetooth que se apaga y se vuelve a
encender se encuentra de nuevo por su ruta; mientras no está, sus players
se comportan como con un dispositivo ausente en el escritorio. Que dos
salidas puedan sonar a la vez depende del teléfono.

## Lo que no está en Android {#not-on-android}

Las superficies de control MIDI, el control remoto, la edición de
etiquetas y carátulas, los archivos de playlists y de páginas de
cartuchos, y la automatización (el musicalizador y Configuración →
Automatización) son por ahora funciones del escritorio. Un teclado conectado a
una tableta funciona como en el escritorio (véase [Teclado](keyboard.md)).
