# Biblioteca

La biblioteca es el conjunto de pistas que Fauste Player conoce: cada archivo
que has añadido a una playlist, a un cartucho o a la propia biblioteca es una
pista, con sus etiquetas, sus marcadores y sus ajustes de sonoridad. La
ventana de la biblioteca las lista, las agrupa en *categorías*, las
encuentra por texto y las envía a las playlists y a la cartuchera. Es una
ventana en el escritorio y en una tableta; un teléfono muestra la
biblioteca en su pestaña **Biblioteca** (véase [En un teléfono](#on-a-phone)).

![La ventana de la biblioteca: la franja de preescucha con la forma de onda, el vúmetro y el fader de escucha de la pista seleccionada, el árbol de categorías con Music (y sus subcategorías Pop y Evening) seleccionada y Marcadores por revisar al final, y la tabla de sus pistas con su categoría y su tipo](../../images/guide/library.png)

## Abrir la ventana {#opening-the-window}

Haz clic en el mosaico **Biblioteca** de la barra superior. Está encendido
mientras la ventana está abierta; haz clic otra vez, o en el botón de cierre
de la ventana, para cerrarla. La ventana flota sobre los players y la
cartuchera y no los bloquea: nada de lo que hay en ella detiene el audio ni
los atajos de la ventana principal.

Muévela por su barra de título y cambia su tamaño desde los bordes y las
esquinas. La barra de título tiene tres botones: **Minimizar la biblioteca**
(solo queda la barra de título, donde estaba la ventana, o en la parte
superior del área principal cuando la ventana está maximizada; **Desplegar
la biblioteca** la expande de nuevo), **Maximizar la biblioteca** (la
ventana llena el área principal; **Restaurar la biblioteca** la devuelve a
su tamaño) y **Cerrar la biblioteca**. Una ventana que no cabría, después de
que la ventana principal se encoja o pase a una pantalla más pequeña, se
devuelve al interior del área principal.

La posición y el tamaño de la ventana, su estado minimizado o maximizado, el
ancho del árbol, las columnas, el orden y la categoría seleccionada se
guardan con la sesión y vuelven en el siguiente arranque. También se guarda
si la ventana está abierta, pero por eso no suena nada por sí solo. El texto
del campo de búsqueda no se guarda.

## Categorías {#categories}

El árbol de la izquierda agrupa las pistas. Arriba, **Todas las pistas**;
luego tus categorías, que pueden contener subcategorías; después, **Sin
categoría**, las pistas que no pertenecen a ninguna categoría; y, solo
mientras alguna pista lo necesite, **Marcadores por revisar** (véase [Cuando
un archivo cambia](#when-a-file-changes)). Cada elemento
muestra su color y un recuento: una categoría cuenta sus propias pistas y las
de sus subcategorías. Una pista pertenece como mucho a una categoría.

- **Haz clic** en un elemento para listar sus pistas en la tabla. Una
  categoría lista también las pistas de sus subcategorías.
- La **flecha** antes de una categoría abre o cierra sus subcategorías. Se
  guarda cuáles están abiertas.
- **Nueva categoría** (＋ sobre el árbol) pide un nombre, un color y un
  *tipo*. Una categoría nueva toma su color y su tipo de la categoría en la
  que se crea; el tipo es el que toma una pista cuando entra en la
  categoría, y cambiarlo después deja las pistas que ya están como están.
  Un nombre no puede estar vacío ni repetir el de una hermana; las
  categorías solo se anidan hasta un límite.
- **Haz clic derecho** en una categoría (o mantén pulsada en una pantalla
  táctil) para **Nueva subcategoría…**, **Cambiar nombre…**, **Color**,
  **Tipo** y **Eliminar…**. En el escritorio el menú también tiene **Vigilar
  carpeta…** (**Cambiar carpeta vigilada…** y **Dejar de vigilar la
  carpeta** una vez que vigila una) y **Quitar pistas que faltan…**; véanse
  [Carpetas vigiladas](#watched-folders) y [Quitar pistas](#removing-tracks).
- **Arrastra** una categoría para moverla: sobre otra categoría para
  anidarla, o al hueco entre dos categorías para colocarla ahí. Una
  categoría no puede ir dentro de sí misma. En una pantalla táctil el árbol
  solo admite toques y se desplaza bajo un dedo; no se puede arrastrar una
  categoría allí.

**Eliminar…** nombra lo que se va con la categoría: sus subcategorías y todas
las pistas que contienen se quitan de la biblioteca. Los archivos nunca se
borran. Si alguna de esas pistas está en uso, una segunda pregunta indica
cuántas entradas de playlist y cuántos cartuchos las usan: **Quitar
igualmente** quita las entradas de sus playlists y vacía los cartuchos. No
se quita nada mientras alguna esté en antena; un aviso dice qué suena.

El mismo árbol está en **Configuración → Biblioteca**, para crear, renombrar,
cambiar el color y el tipo, mover y eliminar categorías sin abrir la ventana
(no tiene *Todas las pistas*, *Sin categoría* ni selección).

![Configuración, Biblioteca: el árbol de categorías con Music, sus subcategorías Pop y Evening, y Jingles, cada una con su número de pistas](../../images/guide/settings-library.png)

## Carpetas vigiladas {#watched-folders}

En el escritorio una categoría puede vigilar una carpeta. Haz clic derecho en
ella, elige **Vigilar carpeta…** y escoge la carpeta. La categoría muestra una
marca ⟳ antes de su recuento; pasa el cursor por encima para leer la
carpeta.

- Todos los archivos de audio de la carpeta y de sus subcarpetas entran en
  la categoría. Las subcarpetas no crean subcategorías.
- Una pista que ya está en otra categoría se queda allí. Una pista que no
  está en ninguna categoría entra en esta. Establecer, cambiar o dejar de
  vigilar nunca mueve una pista: las pistas de una carpeta que ya no se
  vigila se quedan en su categoría.
- Mientras la aplicación funciona, la carpeta se sigue: un archivo nuevo
  entra cuando lleva un momento sin cambios (`library.watch_debounce_ms`, 1
  segundo por defecto, para que un archivo que aún se está copiando no se
  lea a medias). La carpeta también se revisa cada vez que arranca la
  aplicación, para recoger lo que cambió mientras estaba cerrada.
- Un archivo que desaparece de la carpeta (borrado, movido o renombrado)
  deja su pista en la biblioteca, marcada como que falta, con sus
  marcadores. Si el archivo vuelve, la pista vuelve a ser normal por sí
  sola. Un archivo renombrado entra como una pista nueva con su nombre
  nuevo.
- Las carpetas vigiladas se pueden anidar. Un archivo bajo dos de ellas va a
  la categoría de la más interior; dejar de vigilar una de ellas deja la otra
  seguida. La misma carpeta no puede ser vigilada por dos categorías: la
  aplicación dice qué categoría ya la tiene.
- Nada de lo que hace una carpeta vigilada carga ni reproduce una pista.

Si la carpeta no se puede seguir, la marca ⟳ cambia de color y su tooltip
dice por qué: *No se encuentra la carpeta vigilada: …. Se vuelve a buscar.*
(por ejemplo una unidad que no está montada), *No se puede leer la carpeta
vigilada: …. Se vuelve a intentar.*, o *Los cambios en … no se siguen; se
revisa al iniciar y se vuelve a intentar.* (el sistema se negó a
vigilarla). Una carpeta guardada como una ruta que no está completa
(editada a mano) no se vigila en absoluto, y la marca lo dice.

Cada 30 segundos (`tuning.missing_recheck_ms`) la aplicación comprueba que
cada carpeta vigilada sigue ahí, de modo que también se avisa de una unidad
desenchufada mientras funciona. Una carpeta con problemas se vuelve a
intentar a ese ritmo y se revisa en cuanto vuelve. Una carpeta que no está,
no se puede leer o está vacía mientras la biblioteca aún tiene pistas de
ella (una unidad o un recurso compartido que no está montado suele parecer
una carpeta vacía) cuenta como no encontrada. La vigilancia entonces no
marca como faltante ninguna de sus pistas y no quita nada, también cuando la
carpeta está dentro de otra carpeta vigilada.

Si vacías a propósito una carpeta vigilada, el aviso se queda hasta que
vuelvan archivos a ella o dejes de vigilarla, y sus pistas se quedan en la
biblioteca sin marcarse como faltantes. Para limpiarlas, selecciónalas en
la tabla y elige **Quitar de la biblioteca…**, o elige **Volver a analizar**
en ellas: el análisis marca como faltantes las pistas cuyo archivo ya no
está, y **Quitar pistas que faltan…** las quita entonces (si la carpeta
sigue vigilada, su aviso desaparece en la siguiente comprobación).

Una excepción a la regla anterior: una pista que nunca se ha analizado se
marca igualmente como faltante cuando el análisis no encuentra su archivo,
esté donde esté.

Una subcarpeta que no se puede leer se omite y el registro lo dice; los
enlaces a carpetas (enlaces simbólicos a carpetas) no se siguen.

Los archivos nuevos van al análisis por detrás de todo lo que necesita un
player o un cartucho, así que una carpeta con miles de archivos tarda un
rato; el progreso del análisis lo muestra.

El mismo menú y la misma marca están en **Configuración → Biblioteca**. No
están en Android, que no vigila carpetas: allí una categoría puede tomar sus
archivos de una carpeta del dispositivo que vuelves a leer cuando eliges
(véase [Android → Biblioteca](android.md#library)).

## La tabla de pistas {#the-track-table}

La tabla lista las pistas del elemento seleccionado, en las mismas columnas
que las tablas de las playlists (las elegidas en **Configuración → Playlists
→ Columnas de la tabla**, sin la posición `#`) seguidas de **Categoría** y
**Tipo**. Una celda estrecha a la izquierda señala una pista cuyo archivo
falta o no se puede leer. Una pista que espera su análisis se señala a la
derecha de su celda **Título** (así que solo cuando se muestra la columna
Título).

- Arrastra el separador de una cabecera para cambiar el ancho de una
  columna; haz clic en una cabecera para ordenar por ella y otra vez para
  invertir el orden. Se guardan los anchos y el orden.
- **Clic** selecciona una fila, **Ctrl** (**Cmd** en macOS) **clic** añade o
  quita una, **Mayús clic** selecciona las filas entre el último clic y este.
  La selección solo conserva las filas que se listan: una fila que sale de
  la tabla, por una búsqueda, otra categoría o una eliminación, sale de la
  selección. Cada acción se aplica a las filas seleccionadas en el orden de
  la tabla.
- **Añadir archivos** (sobre la tabla) elige archivos de audio y los añade a
  la biblioteca, en la categoría seleccionada (sin categoría cuando está
  seleccionado *Todas las pistas* o *Sin categoría*).

En una pantalla táctil las filas solo admiten toques y se desplazan bajo un
dedo; una pulsación larga abre el menú, y las pistas se envían con los menús
(más abajo), no arrastrando.

## Búsqueda {#search}

El campo sobre la tabla encuentra pistas por texto. Busca en el título, el
artista, el álbum, el artista del álbum, el compositor, el género, el
comentario y el nombre del archivo, sin distinguir mayúsculas ni acentos, y
una pista se lista cuando contiene todas las palabras que escribiste (en
cualquier orden y en cualquiera de esos campos). Busca en el elemento
seleccionado; activa **Toda la biblioteca** para buscar en todas las pistas
sea cual sea la selección. El ámbito se guarda; el texto no. Borra el campo
para volver a ver el elemento entero.

## El menú de la fila {#the-row-menu}

Haz clic derecho en una fila. Si la fila no está seleccionada, pasa a ser la
selección; si no, el menú actúa sobre todas las filas seleccionadas. La
primera línea muestra el título de la pista cuando el menú es para una
pista, y cuántas pistas están seleccionadas cuando es para varias.

| Elemento | Acción |
|---|---|
| Añadir a ▸ | Una playlist: las pistas se añaden al final |
| Añadir a un botón ▸ | Un cartucho de la página visible de la cartuchera: las pistas llenan el cartucho elegido y los siguientes cartuchos vacíos de la página, en el orden de la cuadrícula. Las pistas que no caben se dejan fuera y un aviso las cuenta |
| Mover a la categoría ▸ | Una categoría, o Sin categoría |
| Tipo ▸ | Música, Jingle, Efecto, Anuncio, Voz o Indicativo horario |
| Editar etiquetas… | El editor de etiquetas (una pista) |
| Abrir en el editor externo | Abre tu editor de audio con el archivo de la pista (una pista; véase [Cuando un archivo cambia](#when-a-file-changes)) |
| Marcar como revisada | Quita la marca de *marcadores por revisar* (aparece cuando una pista seleccionada la tiene) |
| Sonoridad… | La ventana emergente de sonoridad (una pista) |
| Volver a analizar | Analiza las pistas otra vez |
| Volver a los marcadores automáticos | Descarta los marcadores manuales |
| Mostrar la ubicación del archivo | Abre la carpeta del archivo (una pista) |
| Quitar de la biblioteca… | Quita las pistas de la biblioteca, tras preguntar (más abajo) |

## Cuando un archivo cambia {#when-a-file-changes}

Preparar material a menudo significa recortar o arreglar un archivo en un
editor de audio. Fauste Player ayuda con las dos mitades.

**Abrir en el editor externo** (el menú de la fila de la biblioteca y de las
filas de las playlists) abre el programa que elegiste en [Configuración →
Biblioteca](settings.md#library) con el archivo de la pista. Actúa sobre una
pista. La entrada aparece atenuada, con el motivo en su información
emergente, cuando no hay ningún editor configurado, cuando el archivo falta
o no se puede leer, o cuando la pista está *sonando*: la pista actual de un
player (sonando o en pausa), una pista que espera para empezar o en CUE, la
pista saliente de un fundido, un cartucho que suena o el cartucho en CUE de
la cartuchera, o la preescucha de la biblioteca mientras suena o está en
pausa. El editor no se ofrece en Android, ni en las versiones Flatpak y
snap, que no pueden abrir otros programas; el resto de esta sección se
aplica también allí.

Fauste Player se da cuenta de cuándo ha cambiado un archivo que conoce (su
tamaño o su fecha de modificación difieren de los del último análisis),
lo haya cambiado quien lo haya cambiado, y lo vuelve a analizar por sí solo:
duración, marcadores, sonoridad, forma de onda, formato y etiquetas. Mira:

- una vez al arrancar, todas las pistas de la biblioteca;
- cuando una carpeta vigilada avisa de un cambio en una pista conocida;
- en la carpeta de cualquier archivo que hayas abierto en el editor, hasta
  que salgas, de modo que un guardado se nota al momento;
- cuando una pista se carga en un player, un cartucho o la preescucha, y
  cuando sus marcadores se abren para editarlos o se arrastran en la forma
  de onda;
- cuando pulsas **Buscar archivos cambiados** en
  [Configuración → Análisis](settings.md#analysis), que revisa toda la
  biblioteca y dice en un aviso cuántos archivos han cambiado.

No hay sondeo periódico. Un archivo fuera de las carpetas vigiladas,
cambiado por otro programa mientras Fauste Player funciona y que no se ha
usado desde entonces, se encuentra en su siguiente uso o con **Buscar
archivos cambiados**.

Una pista cambiada nunca se vuelve a analizar mientras suena: espera a que
se detenga, y sus marcadores y su sonoridad nunca se mueven bajo una fuente
que suena. (**Volver a analizar todas las pistas** es una petición expresa
tuya y no espera). Los editores a menudo guardan borrando y volviendo a
crear el archivo, así que un archivo que desaparece se mira dos veces antes
de marcarlo como no disponible. Un archivo que cambia mientras se analiza
(un editor que todavía lo está guardando) no se usa: se vuelve a analizar
poco después, o en su siguiente cambio, uso o revisión. Una pista no
disponible cuyo archivo vuelve en una carpeta que solo se sigue por el
editor no se recupera por sí sola.

**Marcadores por revisar.** Si el *audio* de un archivo cambió y la pista
tiene marcadores puestos a mano, la pista muestra un icono de aviso ámbar en
su fila (su información emergente nombra los marcadores que se hayan
descartado). Los marcadores manuales que ahora quedan más allá del final del
audio se descartan y su lugar lo ocupa el valor automático; los demás se
quedan como están (cuando se descartó el cue-in o el cue-out, los marcadores
de intro, outro y MIX se mueven dentro del nuevo rango de cue). La marca
nunca impide cargar, poner en CUE ni reproducir la pista. Se quita cuando
eliges **Marcar como revisada** en el menú de la pista (con varias filas
seleccionadas, las quita todas) o cambias cualquier marcador de la pista. El
elemento **Marcadores por revisar** del árbol, después de *Sin categoría*,
lista las pistas marcadas mientras haya alguna (no en un teléfono). Un
cambio solo en las etiquetas o en la carátula actualiza las etiquetas y no
pone ninguna marca, que es también lo que pasa después de editar etiquetas
en Fauste Player. El primer análisis después de una actualización solo pone
la marca si tuvo que descartar un marcador más allá del final del audio.

## Preescucha {#pre-listening}

La barra bajo la barra de título preescucha la pista seleccionada en la
salida **Biblioteca** propia de la biblioteca (elegida en [Configuración →
Salidas de audio](settings.md#audio-outputs)), por ejemplo unos auriculares.
Un teléfono preescucha desde su lista de pistas (véase [En un
teléfono](#on-a-phone)).

La barra muestra:

- **Preescuchar**, **Pausar la preescucha** y **Detener la preescucha**;
- el tiempo restante de la pista en grande, su título, su artista y su
  categoría;
- a la derecha, un vúmetro y un fader de volumen dibujados como en un
  player. El vúmetro lee solo esta preescucha, nunca el vúmetro de un player
  ni el de la cartuchera; haz clic en él para reiniciar su máximo. El fader
  es tu **nivel de escucha** en los auriculares o los monitores: fija lo
  fuerte que suena la preescucha y nunca cambia una pista (su archivo, su
  ganancia de sonoridad ni sus marcadores), un player ni la cartuchera. Se
  queda como lo dejaste cuando la preescucha pasa a otra pista y se guarda
  para el siguiente arranque; su información emergente y la lectura de
  debajo muestran el nivel en dB. Si la salida Biblioteca comparte canales
  con una salida en antena, el fader también fija lo fuerte que suena la
  preescucha allí. La tarjeta de preescucha de un teléfono no tiene fader;
- la forma de onda del archivo entero, tan alta como la de un player, con
  sus marcadores, y una línea `transcurrido / total` debajo.

Preescuchar empieza la pista seleccionada desde su cue-in y la reproduce
hasta el final del archivo. Preescuchar con una preescucha en pausa la
reanuda, y con una que suena no hace nada: Detener y luego Preescuchar
empieza de nuevo desde el cue-in. La normalización de la sonoridad se aplica
como en el CUE de un player.

La forma de onda funciona como en la ventana CUE de un player: haz clic para
saltar (salta un clic o soltar el ratón; arrastrar no), amplía y desplázala
con la rueda, y edita los marcadores y abre **Sonoridad…** desde su menú
(véase [Editar los marcadores](markers-and-mixing.md#editing-markers)).
Mientras la preescucha está detenida, un clic fija en cambio dónde empieza
la siguiente.

**La preescucha sigue a la selección.** Con varias filas seleccionadas, usa
la última en la que hiciste clic.

- Mientras suena, seleccionar otra fila cambia a esa pista desde su cue-in y
  sigue sonando, sin preguntar.
- Mientras está en pausa, la pista nueva se carga en pausa.
- Mientras está detenida, sigue detenida y la barra muestra la pista nueva.
- No seleccionar nada (ninguna fila seleccionada, o la fila seleccionada
  filtrada) deja como está una preescucha en marcha.
- Seleccionar una pista que no se puede reproducir (su archivo falta o no
  se puede leer) detiene la preescucha, y la barra dice que el archivo no
  se puede reproducir.

**Qué la detiene.** Detener; quitar su pista de la biblioteca; cerrar la
ventana (minimizarla no). Nunca suena tras un reinicio.

**Sin una salida Biblioteca** (en el escritorio; Android reproduce en la
salida por defecto, véase [Android → Biblioteca](android.md#library))
Preescuchar está atenuado (pasa el cursor por encima para ver el motivo) y
una línea al lado dice que la preescucha necesita una salida CUE de la
biblioteca, con un enlace que abre Configuración → Salidas de audio en la
fila **Biblioteca**. La barra sigue mostrando la pista, su forma de onda y
sus marcadores.

**En una salida en antena.** La salida Biblioteca puede ser una salida en
antena (el mismo dispositivo y los mismos canales que la Main de un player o
de la cartuchera). Entonces funciona como el CUE de los players y de la
cartuchera (véase [CUE (preescucha)](players.md#cue-pre-listen)): mientras
algo suena allí, Preescuchar pregunta primero (**Preescuchar igualmente** o
**Cancelar**), y una preescucha que no confirmaste se detiene cuando algo
empieza en esa salida.

## Arrastrar a playlists y cartuchos {#dragging-to-playlists-and-carts}

Arrastra las filas seleccionadas (o la fila desde la que empiezas a
arrastrar, cuando no está seleccionada) con el ratón:

- sobre una **pestaña de playlist** de un player: las pistas van al final de
  esa playlist;
- sobre una **tabla de playlist**: las pistas van donde indica la línea
  violeta;
- sobre un **cartucho**: el cartucho recibe la primera pista (un cartucho
  contiene un archivo), sustituyendo su archivo cuando está parado. Un
  cartucho que suena o está en CUE rechaza el soltado, porque un archivo
  nuevo lo detendría en antena.

## Quitar pistas {#removing-tracks}

**Quitar de la biblioteca…** pregunta primero: los archivos no se borran.
Cuando alguna de las pistas se usa, una segunda pregunta cuenta las entradas
de playlist y los cartuchos que las pierden. Una pista que está en antena
(sonando en un player o en la cartuchera) no se puede quitar: no se quita
nada y un aviso dice qué suena.

**Quitar pistas que faltan…** (en el menú de la categoría, en cualquier
categoría) quita las pistas de esa categoría y de sus subcategorías cuyo
archivo falta y que ninguna entrada de playlist ni ningún cartucho usa.
Pregunta primero, con su número; no se toca ningún archivo. Las pistas en
uso se conservan, y una categoría sin ninguna lo dice en un aviso. Úsalo
para limpiar después de que archivos salgan de una carpeta vigilada: la
aplicación nunca los quita por sí sola.

## Desde una playlist {#from-a-playlist}

En el escritorio, el menú de pista de una fila de playlist tiene **Mostrar
en la biblioteca** y **Mover a la categoría ▸**, después de **Sonoridad…**.
*Mostrar en la biblioteca* abre la ventana en la categoría de la pista y
selecciona su fila; *Mover a la categoría* archiva la pista sin abrir la
ventana. Para una pista que no está en la biblioteca, *Mostrar en la
biblioteca* aparece desactivado y *Mover a la categoría* no aparece.

## En un teléfono {#on-a-phone}

Un teléfono no tiene ventana de biblioteca: su pestaña **Biblioteca** se abre
en las categorías (*Todas las pistas*, tus categorías, *Sin categoría* y
**Música del dispositivo**), y un toque en una lista sus pistas con una
búsqueda y **Toda la biblioteca**. Marca pistas para añadirlas a una
playlist, mantén pulsada una fila para su menú, preescucha una pista con su
🎧 y añade la música del dispositivo a una categoría con el **＋** de su
lista. En Android, sin una salida Biblioteca, la preescucha suena en la
salida por defecto. Véase [Android → Biblioteca](android.md#library).
