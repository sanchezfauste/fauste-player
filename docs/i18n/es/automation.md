# Automatización

La automatización mantiene la emisora al aire cuando no hay nadie en el
estudio. Esta versión tiene el **musicalizador**, un player que elige la
música por sí solo, a partir de las reglas que fijas, y que nunca se detiene
por falta de lista, y la columna **Parrilla**, que ejecuta los bloques
horarios del día, con el editor donde los creas.
Los players que ya usas (P1 … Pn) no se tocan: siguen siendo tuyos para los
programas que preparas y emites en directo.

## La columna Parrilla {#the-schedule-column}

La columna **Parrilla** reproduce los **bloques** del día: grupos de
elementos con una hora de inicio (un indicativo horario, cuñas, un
programa). Muéstrala con el mosaico **Parrilla** de la barra superior, igual
que muestras un player.

![La columna Parrilla en marcha, junto a la columna Música: los bloques del día por hora de inicio, cada uno con su indicativo de la emisora (el reloj lo marca como puntual), cuñas y la rotación que el musicalizador reproduce después; la canción del musicalizador al aire y las siguientes](../../images/guide/automation-columns.png)

Su lista es la parrilla, agrupada por bloques: la hora de inicio de cada
bloque y luego sus elementos. Muestra hoy, el último bloque de ayer y el
primero de mañana. Un bloque cambiado para un día lo indica. El elemento al
aire y el siguiente se marcan como en un player; los elementos que ya se
emitieron aparecen atenuados, y un elemento que no se emitió (**No
emitido**) aparece tachado. Un reloj marca un elemento **puntual**, que solo
suena a su hora.

Encima de la lista:

- **Iniciar** pone en marcha la parrilla: cada bloque suena a su hora y el
  musicalizador suena entre bloques. Nada se pone en marcha por sí solo:
  después de cada arranque de la aplicación, la automatización está
  detenida hasta que pulsas Iniciar.
- **Detener tras la actual** deja que termine lo que está al aire, y luego
  ya no suena nada por sí solo.
- **En marcha** o **Detenida** dice en qué estado está.

Para cambiar lo que suena a continuación, haz doble clic en un elemento, o
haz clic derecho en él y elige **Que suene a continuación**; sigue a la
canción que está al aire. **Volver al reloj**, en el mismo menú, deja que la
parrilla vuelva a elegir según la hora.

## Editar la parrilla {#editing-the-schedule}

**Editar parrilla**, en la columna Parrilla, abre el editor. Tiene tres
pestañas, y **Rotaciones…** te lleva a Configuración → Automatización, donde
se crean las rotaciones.

![El editor de parrilla, pestaña Plantillas: las plantillas Base y Weekend a la izquierda; los bloques de Base a la derecha, cada uno con su inicio, sus elementos, Añadir de categoría y la rotación de la música que va después](../../images/guide/schedule-templates.png)

- **Plantillas.** Una plantilla es un día de bloques; una instalación nueva
  tiene una, "Base", para todos los días. Crea, duplica, renombra o elimina
  plantillas a la izquierda; los bloques de la elegida están a la derecha.
  Escribe el inicio de un bloque como `HH:MM:SS` (una hora que no existe se
  queda en el campo con una nota, y no cambia nada). Arrastra pistas de la
  biblioteca a un bloque para añadirlas; **Añadir de categoría** añade un
  elemento que reproduce una pista de esa categoría, una distinta cada vez;
  **Música después** elige la rotación que el musicalizador reproduce
  después del bloque (la rotación por defecto si no se elige ninguna). Cada
  elemento se puede marcar como **Puntual**, mover o quitar. Cuando los
  elementos de un bloque duran más que el tiempo hasta el bloque siguiente,
  el bloque dice en cuánto, en ámbar; aun así puedes dejarlo así.
- **Semana tipo.** Qué plantilla usa cada día de la semana.

  ![La pestaña Semana tipo: Base de lunes a viernes, Weekend el sábado y el domingo](../../images/guide/schedule-week.png)

- **Días.** Elige una fecha en el calendario (las fechas con cambios están
  en ámbar). Una fecha puede usar otra plantilla distinta de la de su día de
  la semana (un festivo en jueves puede usar la del fin de semana). Sus
  bloques se pueden cambiar solo para esa fecha: el cambio queda marcado, y
  **Volver a la plantilla** lo deshace. **Este día no** quita un bloque solo
  de esa fecha, y **Añadir bloque este día** añade uno. Los cambios
  posteriores en la plantilla siguen llegando a los bloques que no cambiaste
  en esa fecha.

  ![La pestaña Días: el calendario con hoy elegido y las fechas con cambios en ámbar; el primer bloque de hoy tiene una cuña más, marcado como "cambiado para este día", con Volver a la plantilla](../../images/guide/schedule-days.png)

## El musicalizador {#the-music-filler}

El musicalizador es la columna **Música**. Está oculta hasta que la
muestras: haz clic en el mosaico **Música** de la izquierda de la barra
superior. Se sitúa a la izquierda de P1.

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
**Música**, como para cualquier player. No tiene número, ni atajos de
teclado ni asignaciones MIDI, y el control remoto no la ve.

Pulsa **Play** para ponerla en marcha y **Detener tras la actual** para
dejar que termine la canción que está al aire y se detenga. Como todos los
players, nunca se pone en marcha por sí sola, ni siquiera después de un
reinicio.

Encima de su lista, donde un player tiene las pestañas de sus playlists, la
columna Música muestra la **rotación** que reproduce. Elige otra allí: las
canciones que tenía en cola se sustituyen y las que añadiste se quedan.
Cuando la rotación no encuentra nada que reproducir (sus categorías están
vacías, o faltan todos los archivos), la barra lo dice.

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

Las rotaciones se crean en Configuración → Automatización:

![Configuración, Automatización: las rotaciones Música (la por defecto) y Hits, la Música seleccionada con su único paso sobre toda la música, sus cantidades Como mínimo y Como máximo, y las reglas del musicalizador](../../images/guide/settings-automation.png)

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
la canción que baja con un fundido cuando empieza el bloque pierde como
mucho 30 segundos, y no empieza ninguna canción que fuera a sonar menos de
45 segundos. Cuando ninguna canción de la rotación puede cumplir las dos
cosas, suena la que menos pierde. Por ahora, los dos valores solo se pueden
cambiar en el archivo de configuración (`automation.max_fade_cut_secs` y
`automation.min_song_play_secs`).

Un **Indicativo horario** es un tipo de pista propio (Biblioteca → Tipo ▸
Indicativo horario), para la identificación de la emisora a la hora en
punto.
