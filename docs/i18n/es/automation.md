# Automatización

La automatización mantiene la emisora al aire cuando no hay nadie en el
estudio. Esta versión tiene el **musicalizador**, un player que elige la
música por sí solo, a partir de las reglas que fijas, y que nunca se detiene
por falta de lista, y la columna **Parrilla**, que ejecuta los bloques
horarios del día, hechos a partir de plantillas en Configuración.
Los players que ya usas (P1 … Pn) no se tocan: siguen siendo tuyos para los
programas que preparas y emites en directo.

## La columna Parrilla {#the-log-column}

La columna **Parrilla** reproduce los **bloques** del día: grupos de
elementos con una hora de inicio (un indicativo horario, cuñas, un
programa). Muéstrala con el mosaico **Parrilla** de la barra superior, igual
que muestras un player.

![La columna Parrilla en marcha, junto a la columna Musicalizador: los bloques del día por hora de inicio, cada uno con su indicativo de la emisora (el reloj lo marca como de hora fija), cuñas y la rotación que el musicalizador reproduce después; la canción del musicalizador al aire y las siguientes](../../images/guide/automation-columns.png)

Su lista parece una playlist, agrupada por bloques. Cada bloque empieza con
una fila de cabecera: su hora de inicio, cuántos elementos tiene y cuánto
duran, "cambiado para este día" cuando lo cambiaste hoy y, en ámbar, cuánto
se pasa del bloque siguiente. Luego vienen sus elementos, con la **Hora** a
la que está previsto que empiece cada uno. La lista muestra hoy, el último
bloque de ayer y el primero de mañana. El elemento al aire y el siguiente se
marcan como en un player; los elementos que ya se emitieron aparecen
atenuados, y un elemento que no se emitió (**no emitido**) aparece tachado.
Un reloj marca un elemento de **hora fija**, que solo suena a su hora. Una
**pista de categoría** muestra la pista elegida para ella en cuanto pasa a
ser la siguiente (o viene después de la siguiente en su bloque), con la
categoría detrás: esa es la pista que suena. Si la categoría no tiene
ninguna pista que pueda sonar (no tiene ninguna asignada, o faltan sus
archivos), la fila lo dice en ámbar, y el elemento no se emitirá. Un
elemento cuyo archivo resulta ilegible cuando va a sonar se salta y también
se marca como no emitido. Las pistas se asignan a una categoría en la
biblioteca. La fila del **musicalizador** dice hasta cuándo suena, y se
muestra al aire mientras el musicalizador suena para la Parrilla. Un
siguiente cuya hora ya ha pasado muestra la hora en ámbar. Arrastra la línea
entre dos cabeceras de columna para cambiar el ancho de las columnas, como
en una playlist; las ventanas de plantilla y de día usan los mismos anchos.

### Cambiar la parrilla de hoy {#changing-todays-log}

La Parrilla se cambia como una playlist, y el cambio es solo para ese día:
la plantilla se queda como está.

- Los botones de la parte inferior de la columna: **+ Pista…** abre la
  biblioteca, para arrastrar pistas a la lista; **+ Pista de categoría…** y
  **+ Musicalizador (rotación)…** preguntan cuál y la añaden justo debajo de
  la fila que seleccionaste (haz clic en una fila para seleccionarla;
  selecciona una cabecera de bloque para añadir al principio de ese bloque),
  o, sin nada seleccionado o debajo de una fila que ya no puede cambiar, al
  final del próximo bloque; **+ Bloque horario…** pide una hora, con la
  primera libre ya puesta. Cuando la columna es demasiado estrecha para
  todos, los que no caben están detrás de **⋯**.
- Arrastra pistas de la biblioteca, o archivos de audio de tu gestor de
  archivos, a una fila para ponerlos delante de ella (o detrás, en la mitad
  inferior de la fila, como en una playlist: así van al final de un bloque),
  y arrastra filas dentro del mismo día para moverlas. La fila que añades
  queda seleccionada y a la vista, así que la siguiente que añadas va debajo.
- Haz clic derecho en una fila para **Hora fija** y **Quitar de la parrilla
  de hoy**. Haz clic (o clic derecho) en una cabecera de bloque para añadir
  una **Pista…**, una **Pista de categoría…** o un **Musicalizador
  (rotación)…** al final de ese bloque, o para **Cambiar hora…**, **Quitar
  bloque** y, en un bloque cambiado, **Volver a la plantilla**.
- Mientras arrastras sobre una cabecera de bloque, la línea que aparece
  debajo indica que lo sueltas al principio de ese bloque; para poner filas
  al final del bloque de arriba, suéltalas en la mitad inferior de su última
  fila.
- Lo que ya sonó, el elemento al aire y los días pasados no se pueden
  cambiar. Un cambio en el bloque al aire tiene efecto después del elemento
  al aire.

La columna Parrilla tiene la cabecera y los botones de un player, y
funcionan como en un player:

- **Reproducir** reproduce el elemento marcado como siguiente y pone en
  marcha la automatización: cada bloque suena entonces a su hora, y el
  musicalizador suena entre bloques. Cuando el siguiente lo eligió el reloj
  y su bloque empieza más tarde, la música suena hasta entonces. Nada se pone
  en marcha por sí solo: después de cada arranque de la aplicación, la
  automatización está detenida hasta que pulsas Reproducir.
- **Pausa**, **Stop** y **Stop con fundido** actúan sobre lo que está al
  aire para la Parrilla (su elemento, o la canción del musicalizador entre
  bloques) y detienen la automatización. Reproducir después de Pausa sigue
  desde donde estaba; mientras está en pausa, el siguiente no se mueve, y al
  reanudar el bloque siguiente mantiene su hora. Reproducir después de un
  Stop durante la música vuelve a poner música, y el bloque siguiente
  empieza a su hora.
- **Stop al final** deja que termine lo que está al aire, y luego la
  automatización se detiene. Si está sonando la música y antes llega la hora
  de un bloque, el bloque empieza puntual y la automatización se detiene
  después de su primer elemento.
- **Pista anterior** no hace nada en la Parrilla.
- **SINGLE** hace manual la Parrilla: Reproducir reproduce el elemento
  siguiente, y se detiene después de él; ningún bloque empieza por sí solo.

Mientras el musicalizador suena entre bloques, la cabecera de la Parrilla
dice **Musicalizador · *rotación*** (la rotación que reproduce) y sus
botones siguen funcionando; la canción, su medidor y su forma de onda están
solo en la columna Musicalizador.

Cuando la automatización se detiene después de un elemento, el siguiente
queda en el elemento de debajo del que sonó, y Reproducir lo reproduce, sea
la hora que sea: eliges tú. Cuando se detiene durante la música, Reproducir
vuelve a poner música hasta el bloque siguiente, salvo que hayas elegido
otro siguiente. Para cambiar lo que suena a continuación, haz doble clic en
un elemento, o haz clic derecho en él y elige **Que suene a continuación**.
**Volver al reloj**, en el mismo menú, pone como siguiente lo que dice la
hora.

## Plantillas y días {#templates-and-days}

La Parrilla reproduce cada día a partir de una **plantilla**: un día de
bloques. Una instalación nueva tiene una, "Base", para todos los días. Las
plantillas se hacen en Configuración → **Automatización**, en **Plantillas
y días** (no en Android); la columna Parrilla solo cambia el día de hoy,
como se ha visto arriba.

![Configuración, Automatización, Plantillas y días: las plantillas Base y Weekend con Editar…, Nueva plantilla, Duplicar, Eliminar y el nombre; la semana tipo, Base de lunes a viernes y Weekend el sábado y el domingo; el mes con hoy elegido, las fechas con cambios en ámbar, la plantilla de la fecha y Editar día…](../../images/guide/settings-plans.png)

- **Plantillas.** Elige una y pulsa **Editar…** (o haz doble clic en ella)
  para abrirla; **Nueva plantilla**, **Duplicar** y **Eliminar** están al
  lado, y su nombre debajo.
- **Semana tipo.** Qué plantilla usa cada día de la semana.
- **Días.** Elige una fecha en el calendario (las fechas con cambios están
  en ámbar). Una fecha puede usar otra plantilla distinta de la de su día de
  la semana (un festivo en jueves puede usar la del fin de semana).
  **Editar día…** abre esa fecha, para cambiar sus bloques solo para esa
  fecha; las fechas pasadas no se pueden cambiar.

Una plantilla o una fecha se abre en su propia ventana, que cierra
Configuración. Es la misma lista que la columna Parrilla, y se cambia igual:
los botones de su parte inferior (Pista…, Pista de categoría…, Musicalizador
(rotación)…, Bloque horario…, que añaden debajo de la fila que seleccionas),
pistas arrastradas desde la biblioteca (o archivos desde tu gestor de
archivos) a una fila, filas arrastradas para moverlas, el menú de clic
derecho de las filas y el menú de las cabeceras de bloque. Nunca suena, y
puede quedarse abierta junto a la biblioteca. La ventana de hoy muestra lo
que sonó y lo que suena, como la Parrilla; la ventana de un día se cierra a
medianoche, cuando ese día ya no puede cambiar.

![Una plantilla abierta en su ventana: los bloques de la plantilla Base por hora de inicio, cada uno con su indicativo de la emisora, sus cuñas y el musicalizador después, y Añadir en la parte inferior](../../images/guide/template-popup.png)

Una **pista de categoría** en una plantilla reproduce una pista de esa
categoría, una distinta cada vez. El elemento **Musicalizador** elige la
rotación que el musicalizador reproduce después del bloque (la rotación por
defecto cuando el bloque no tiene ninguna). Cuando los elementos de un
bloque duran más que el tiempo hasta el bloque siguiente, su cabecera dice
en cuánto, en ámbar; aun así puedes dejarlo así.

Un cambio en una fecha queda marcado como "cambiado para este día" en su
bloque, y **Volver a la plantilla**, en el menú de la cabecera del bloque,
lo deshace. Los cambios posteriores en la plantilla siguen llegando a los
bloques que no cambiaste en esa fecha. Un cambio en una plantilla hecho
mientras uno de sus bloques está al aire deja ese bloque como está para hoy
(muestra "cambiado para este día"); el cambio le llega a partir del día
siguiente.

## El musicalizador {#the-music-fill}

El musicalizador es la columna **Musicalizador**. Está oculta hasta que la
muestras: haz clic en el mosaico **Musicalizador** de la izquierda de la
barra superior. Se sitúa a la izquierda de P1.

Tiene el aspecto de un player y funciona como tal: el reloj grande, la forma
de onda, los botones de transporte, el fader y la lista de pistas. La
diferencia está en la lista:

- **se llena sola** a partir de una **rotación** (más abajo), siempre con
  una media hora de adelanto (el *margen por delante*, en Configuración →
  Automatización);
- **se recorta sola**: conserva las últimas 50 pistas que reprodujo y quita
  las más antiguas, de modo que puede funcionar durante semanas;
- antes de un bloque de la parrilla puede **cambiar canciones que eligió
  ella misma** por otras cuya duración encaje en el tiempo que queda (véase
  más abajo); las canciones que añadiste tú nunca se tocan;
- todavía puedes **cambiarla a mano**: arrastrar pistas desde la
  biblioteca, reordenar filas, quitar una canción que no quieres. Una fila
  que añades se marca con una mano y nunca se sustituye; una fila que quitas
  no cuenta como reproducida.

Sus salidas se ajustan en Configuración → Salidas de audio, en la fila
**Musicalizador**, como para cualquier player. No tiene número, ni atajos de
teclado ni asignaciones MIDI, y el control remoto no la ve.

Pulsa **Reproducir** para ponerla en marcha y **Stop al final** para dejar
que termine la canción que está al aire y se detenga. Como todos los
players, nunca se pone en marcha por sí sola, ni siquiera después de un
reinicio. Mientras suena para la Parrilla entre bloques, detenerla o
ponerla en pausa también detiene la automatización, y su Stop al final
detiene la automatización cuando termina la canción; si antes llega la hora
de un bloque, el bloque suena y la marca se quita. Después de poner en
pausa la Parrilla durante la música, Reproducir en la columna Musicalizador
reproduce la canción por su cuenta: el Reproducir de la Parrilla ya no la
reanuda, y su Stop ya no la detiene.

Encima de su lista, donde un player tiene las pestañas de sus playlists, la
columna Musicalizador muestra la **rotación** que reproduce. Elige otra
allí: las canciones que tenía en cola se sustituyen y las que añadiste se
quedan. Cuando la rotación no encuentra nada que reproducir (sus categorías
están vacías, o faltan todos los archivos), la barra lo dice.

## Rotaciones {#rotations}

Una **rotación** es una lista corta de **pasos** que el musicalizador
reproduce por turnos, y luego otra vez desde el primero. Cada paso indica de
dónde salen sus pistas (una categoría de la biblioteca con sus
subcategorías, o todas las pistas de un tipo) y cuántas reproduce cada vez,
como un rango:

| Paso | Reproduce |
|---|---|
| Éxitos actuales | 2–3 |
| Indicativos de la emisora | 1 |
| Clásicos | 0–1 |

Con un rango, un indicativo de la emisora llega después de dos canciones
una vez y después de tres la siguiente. Un rango desde **0** permite que un
paso se salte algunos turnos: con Clásicos en 0–1, una canción clásica suena
después del indicativo unas veces y otras no. El máximo siempre es al menos
1. Una rotación es la **por defecto**: una instalación nueva tiene una,
**Música**, que reproduce toda tu música.

Un paso cuya categoría no tiene ninguna pista que pueda sonar (una
categoría vacía, o archivos que faltan) reproduce en su turno una canción
de la rotación por defecto (cualquier música, si el paso es de la propia
rotación por defecto), de modo que los demás pasos mantienen su lugar: un
indicativo de la emisora sigue llegando entre canciones.

Las rotaciones se crean en Configuración → Automatización:

![Configuración, Automatización: las rotaciones Música (la por defecto) y Hits, la Música seleccionada con su único paso sobre toda la música, sus cantidades Como mínimo y Como máximo, y el principio de Plantillas y días](../../images/guide/settings-automation.png)

## Cómo elige {#how-it-chooses}

Dentro de un paso, el musicalizador prefiere las pistas que llevan más
tiempo sin emitirse, con un poco de azar entre ellas para que el orden nunca
se convierta en un ciclo fijo. También mantiene separados:

- **la misma pista:** puede volver a sonar cuando hayan pasado 3 horas o
  cuando haya sonado desde entonces la mitad de las demás pistas de su paso,
  lo que ocurra primero;
- **el mismo artista:** no en 30 minutos.

Cuenta todo lo que salió al aire, desde el musicalizador, un player o la
cartuchera; la preescucha no. Cuando una categoría pequeña no puede cumplir
las reglas, el musicalizador cede antes que quedarse en silencio: primero en
el artista y luego en la pista. Configuración → Automatización tiene todos
los valores, y es donde creas, renombras y cambias las rotaciones (véase
[Configuración](settings.md#automation)).

Antes de un bloque de la parrilla, el musicalizador también elige sus
últimas canciones por su duración, para que la música termine limpiamente:
la canción que baja con un fundido cuando empieza el bloque sigue sonando a
la hora del bloque y pierde como mucho 30 segundos después de ella, y no
empieza ninguna canción que fuera a sonar menos de 45 segundos. También
mantiene las reglas de artista y de pista siempre que puede. Cuando ninguna
canción de la rotación puede cumplir las dos cosas, suena la que menos
pierde, así que a veces se pueden oír unos segundos de una canción. Una
pausa de la música mueve las canciones que van detrás, y el musicalizador
las vuelve a elegir. El musicalizador también evita las canciones, y los
artistas, que un bloque reproduce poco después. Por ahora, los dos valores
solo se pueden cambiar en el archivo de configuración
(`automation.max_fade_cut_secs` y `automation.min_song_play_secs`).

Un **Indicativo horario** es un tipo de pista propio (Biblioteca → Tipo ▸
Indicativo horario), para la identificación de la emisora a la hora en
punto.
