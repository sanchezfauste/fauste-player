# Players

Cada columna es un player. Los players son independientes: cada uno tiene
sus propias pestañas de playlist, transporte, volumen y salidas.

![El player 1 sonando: su cabecera, carátula, título, siguiente pista, transporte, cuenta atrás, vúmetro, fader y forma de onda](../../images/guide/player.png)

## Cabecera {#header}

| Elemento | Significado |
|---|---|
| `P1` … `Pn` | Número del player (la tecla numérica que lo reproduce) |
| Punto y etiqueta de estado | **Sonando** (rojo), **En pausa** (ámbar), **Detenido** (gris) |
| Distintivo **Mezcla** / **Fundido** | Hay en curso una mezcla hacia la siguiente pista o un stop con fundido |
| Distintivo **Stop al final** | El player se detiene cuando acaba la pista actual |
| Distintivo **Repetir** / **Parar tras la pista** | La pista actual se repite, o detiene el player al acabar, por su propia marca en el menú de la playlist. Pasa el puntero por encima para ver la frase completa. El botón **Stop al final** del propio player tiene prioridad: mientras está activo, solo se ve su distintivo |
| **BP** / **DSD** | **BP** se enciende mientras la pista actual llega sin cambios a su dispositivo Main. **DSD** lo sustituye mientras una pista DSD sale como DSD, sin cambios (consulta [Salida bit-perfect](bit-perfect.md)). **Otras silenciadas** aparece a su lado cuando ese flujo DSD deja fuera de la salida a las demás fuentes |
| **SINGLE** \| **CONT** | Modo de reproducción (ver más abajo): un único control partido en dos; la mitad iluminada es el modo activo |
| **CUE** | Preescucha de la siguiente pista en la salida CUE |

## Fila de información {#info-row}

- **Carátula** de la pista que suena, o un disco de vinilo en su lugar.
- **Título y artista** de la pista que suena. Un player detenido muestra la
  pista que Play pondrá en marcha (su siguiente), con su carátula, su
  duración y su forma de onda, preparada en el cue-in o donde hayas hecho
  clic en su forma de onda.
- Una pista reproducida o cargada antes de que acabe su análisis ya tiene su
  duración si la cabecera del archivo la indica: la cuenta atrás,
  `transcurrido / total` y el clic para saltar funcionan al instante, sobre
  una línea plana hasta que la forma de onda está lista. Si la cabecera no
  la guarda (archivos AAC en bruto, MP3 sin trama de duración, Matroska y
  WebM), el total muestra «—» y no se puede hacer clic en la forma de onda
  hasta que termina el análisis.
- **Vúmetro estéreo** (en la columna de la derecha del player, junto al
  fader; ocupa la fila de información y el transporte): el nivel que entrega
  el player, después de su volumen.
  - Una barra continua por canal, en la escala de la norma del tipo de
    medidor (por defecto, el medidor de pico digital: de −60 a 0 dBFS, con
    los 20 dB superiores ocupando la mitad de la altura). La escala es una
    regla a cada lado de las barras, rotulada en las unidades del propio
    medidor en ambos lados. Su parte superior y su parte inferior (en el
    medidor digital, el suelo de la escala) siempre están marcadas; cada
    rótulo tiene una marca en cada regla, y las marcas más cortas entre los
    rótulos funcionan como las de una regla de medir: espaciadas de forma
    regular en valores redondos, cada 1 dB en el medidor EBU y cada 5 dB por
    debajo de −20 en el digital cuando el medidor es lo bastante alto, y
    cada 2, 2,5, 5 o 10 dB (o ninguna) donde es demasiado corto para ellas.
    En el medidor digital (y en el personalizado), un medidor alto rotula
    más valores: cada 1 dB de −20 a 0, y cada 5 dB entre las marcas de 10 dB
    por debajo de −20 (−45, −55), siempre espaciados de forma regular y solo
    donde caben sin que los rótulos se toquen; los demás tipos de medidor
    mantienen los rótulos que da su norma. El nivel de alineación
    (−18 dBFS en el medidor digital) es una marca blanca más gruesa en
    ambas reglas. No se dibuja nada encima de las barras ni entre ellas, así
    que lo que ves en las barras es solo el nivel, la retención del pico y
    los colores.
  - La barra es verde, amarilla desde el nivel de aviso (−9 dBFS) y roja
    desde el nivel de peligro (−3 dBFS). Los demás tipos de medidor se
    ponen rojos donde lo hace su escala (desde 0 VU, o desde el máximo
    permitido en un PPM).
  - Los medidores K-System muestran dos secciones: la barra sólida es el
    nivel medio (RMS) y la parte más tenue por encima llega hasta el pico.
    Sus colores son los del K-System: verde por debajo de 0, ámbar de 0 a
    +4 y rojo por encima.
  - Los medidores de pico digital, K-System y personalizado mantienen
    encendido un momento el nivel más alto como una línea (la retención del
    pico; su duración es **Retención del pico** en
    [Configuración → Vúmetros](settings.md#meters), y 0 la desactiva). Los
    medidores PPM EBU, PPM DIN y VU no tienen retención.
  - El número de arriba es el nivel más alto desde que empezó la entrada,
    en dBFS, en rojo en la zona de peligro. Se mantiene tras un stop y
    vuelve a empezar cuando suena una entrada (la siguiente o la misma otra
    vez), o cuando haces clic en él.
  - El número de abajo es la sonoridad en LUFS (EBU R128), en verde a
    ±1 LU del objetivo (−23 LUFS).
  - El tipo de medidor y todos los niveles se cambian en
    [Configuración → Vúmetros](settings.md#meters).
  - **Lecturas por encima de 0 dBFS.** El vúmetro muestra lo que entrega el
    player, y eso puede superar la escala completa. La barra se detiene en
    lo alto de la escala, así que el mismo rojo indica 0 dBFS y cualquier
    valor superior; solo el número de arriba dice cuánto, con su signo (por
    ejemplo `+3.5`).
    - Un archivo puede traer niveles por encima de la escala completa (un
      archivo en coma flotante, o uno con pérdida cuyos picos decodificados
      la superan).
    - Convertir la frecuencia puede crear picos entre las muestras: una
      señal que toca 0 dBFS marca unos +3 dBFS tras pasar de 44,1 a 48 kHz.
      La opción de true peak también lee esos picos.
    - El vúmetro mide cada player por separado, no la suma en el
      dispositivo: dos players en una misma salida pueden sumar por encima
      de la escala completa sin que ninguno de los dos vúmetros lo muestre.
    - Nada en el player añade ganancia por encima del 100 %. Un dispositivo
      de enteros recorta en la escala completa; uno de coma flotante recibe
      el nivel tal cual y lo recorta el sistema de sonido o el controlador.
- **Fader de volumen** (a la derecha del vúmetro, igual de alto): arrástralo
  o usa la rueda del ratón, un paso por muesca. La información emergente
  muestra el nivel en dB; arriba del todo es 0 dB y abajo del todo, silencio.
- **Título, artista** y la línea de la **siguiente**, con un cuadrado verde.
  Mientras el CUE está activo, la posición de la preescucha se ve en azul.

## Transporte {#transport}

| Botón | Acción |
|---|---|
| **Play / Siguiente** (grande) | Detenido: inicia la siguiente pista. Sonando: hace un fundido hacia la siguiente pista (la duración del fundido se ajusta en [Configuración](settings.md)). En pausa: reanuda. Si la siguiente es la pista que está sonando, Play la vuelve a empezar con el fundido habitual. |
| **Stop** | Se detiene al momento (con una rampa corta para evitar el chasquido) |
| **Stop con fundido** | Baja el volumen con un fundido y se detiene |
| **Pausa** | Pausa o reanuda; parpadea en ámbar mientras está en pausa |
| **Stop al final** (un triángulo de play y un cuadrado) | Se detiene cuando acaba la pista actual, una sola vez. En modo SINGLE solo está disponible mientras la pista actual se repite: termina la repetición cuando acaba la pasada en curso. Para detenerse tras una pista cada vez que suene, o para repetir una pista, usa su menú en la playlist (consulta [Playlists](playlists.md)) |
| **Anterior** (una barra y dos triángulos) | Sonando: hace un fundido de vuelta a la pista que este player reprodujo antes, igual que Siguiente. Púlsalo otra vez para seguir retrocediendo. La pista que dejas pasa a ser la siguiente. |
| **Reiniciar** (una barra y un triángulo) | Vuelve al principio de la pista actual (su cue-in). Un player en pausa sigue en pausa. |

Los botones que no pueden actuar en ese momento aparecen atenuados: Stop y
Reiniciar sin nada cargado, Pausa y Stop con fundido con el player detenido,
y Anterior sin una pista anterior o durante un fundido. Un player recuerda
las últimas 50 pistas que ha reproducido (`players.history_len` en
`config.json`, de 0 a 1000).

## Modos {#modes}

- **CONT (continuo):** en el punto MIX el player inicia la siguiente pista y
  la solapa con el final de la actual. Consulta
  [Marcadores y mezcla](markers-and-mixing.md).
- **SINGLE:** cada pista se detiene al acabar. *Stop al final* no está
  disponible en este modo, porque cada pista ya se detiene, salvo mientras
  la pista actual se repite: entonces termina la repetición cuando acaba la
  pasada en curso.

## Cuenta atrás {#countdown}

El número grande es el tiempo que falta hasta el final de la pista (su
cue-out), con décimas. El tiempo transcurrido y el total están en la fila
de debajo de la forma de onda, a la derecha. Durante los últimos segundos
antes del final (10 por defecto, se ajusta en Configuración) la cuenta atrás
parpadea en rojo.

## Forma de onda {#waveform}

- La parte ya reproducida se dibuja con el color de la forma de onda; el
  resto, más tenue.
- El contorno muestra los picos, de forma tenue; el cuerpo sólido de dentro
  es el nivel medio (RMS). En una pista fuerte los picos llenan la altura, y
  el cuerpo sigue mostrando dónde la pista suena más baja o más alta.
- Una zona sombreada en azul al principio marca la **intro**, y un
  distintivo la cuenta hacia atrás. La intro solo aparece si se ha fijado.
- Una zona sombreada en naranja al final marca el **outro**, con su propia
  cuenta atrás.
- Una línea discontinua ámbar con la etiqueta **MIX** marca dónde empieza la
  siguiente pista en modo continuo. En modo single aparece atenuada.
- Se dibuja el archivo entero. El principio y el final en silencio que la
  reproducción se salta (antes del cue-in y después del cue-out) se dibujan
  más oscuros, con una línea fina donde empieza y acaba la reproducción. Con
  **Usar cue-in y cue-out** desactivado (Configuración → Players) no hay
  nada más oscuro y las dos líneas se atenúan: la reproducción va del
  principio al final del archivo, y en esta guía el cue-in y el cue-out
  significan esos dos extremos.
- Pasa el puntero por encima para ver el tiempo bajo él. **Haz clic para
  saltar** allí. En un player detenido, un clic elige dónde empieza **Play**
  la siguiente pista: el cabezal y la cuenta atrás se mueven allí, y no
  suena nada hasta que pulsas Play. Elegir otra siguiente pista, moverla o
  quitarla, o Stop, vuelve al cue-in; lo mismo ocurre con cualquier otra
  forma de iniciar una pista, y Reiniciar, Anterior y el avance automático
  siempre usan el cue-in. Un clic antes del cue-in (en el principio más
  oscuro) elige el cue-in. Un clic en el cue-out o después (en la cola más
  oscura) cancela una elección anterior: Play empieza en el cue-in. Un clic
  es pulsar y soltar sin mover el puntero más de unos pocos píxeles.
- **Pulsar y arrastrar** desplaza la vista ampliada a lo largo de la pista,
  como si la agarraras. Arrastrar nunca salta, y sin zoom no hace nada.
  Arrastrar con Alt sigue editando marcadores.
- **Rueda del ratón** sobre la forma de onda: amplía y reduce alrededor del
  puntero, hasta el máximo detalle que tiene el análisis. **Mayús+rueda** (o
  una rueda lateral) se desplaza a lo largo de la pista. Con zoom, la vista
  sigue la posición de reproducción, salvo durante 10 segundos después de
  ampliar o desplazarla (`ui.follow_current_grace_secs`; 0 desactiva el
  seguimiento). **Vista completa**, en la esquina superior derecha, reducir
  el zoom del todo o una pista nueva vuelven a mostrar la pista entera.

## CUE (preescucha) {#cue-pre-listen}

Pulsar **CUE** (o **Preescuchar en CUE** en el menú de una pista) reproduce
la pista en la salida CUE del player, por ejemplo unos auriculares, sin
tocar la salida al aire, y abre una pequeña **ventana de CUE** para ese
player. Puede haber varias ventanas abiertas, una por player. Consulta
[Configuración](settings.md) para elegir el dispositivo de CUE. Un player
necesita una salida Cue que no sea su salida Main: sin ella, **CUE** y
**Preescuchar en CUE** aparecen atenuados, y al pasar el puntero por encima
se explica por qué.

![La ventana de CUE del player 4, preescuchando su siguiente pista](../../images/guide/cue-window.png)

La ventana muestra:

- el título y el artista;
- la forma de onda del archivo entero con la posición del CUE. Funciona como
  la del player: clic para saltar, zoom con la rueda, arrastrar para
  desplazarse, **Vista completa**, las cuentas atrás de la intro y el outro,
  y los marcadores de intro, outro y MIX, que se editan aquí igual que en el
  player (consulta [Marcadores y mezcla](markers-and-mixing.md)). Un CUE
  reproduce el archivo entero, así que no hay nada más oscuro, el cue-in y
  el cue-out son líneas atenuadas y el outro cuenta hasta el final del
  archivo. Su zoom es independiente: la forma de onda del player no se
  mueve. Mientras suena el CUE, una vista ampliada sigue su posición como lo
  hace la del player; un CUE en pausa mantiene la vista que hayas fijado,
  para que puedas ampliar y colocar marcadores. Un CUE iniciado después de
  cerrar su ventana, o en otra pista, muestra el archivo entero;
- el tiempo transcurrido y el que falta hasta el final del archivo (un CUE
  reproduce archivos enteros);
- **Pausar CUE** / **Reanudar CUE**, **Parar CUE** y **Marcar como
  siguiente**. **Marcar como siguiente** convierte la pista en CUE en la
  siguiente del player y deja el CUE sonando. Aparece atenuado si la pista
  ya es la siguiente. Si la pista en CUE es la que está sonando, se
  reproduce una vez más cuando acaba la pasada en curso.

Mientras el CUE está en pausa, su botón de pausa (que muestra **Reanudar
CUE**) parpadea en ámbar, como el del propio player.

Un salto en un CUE en pausa lo mantiene en pausa. El botón de cerrar de la
ventana, o **Parar CUE**, detiene el CUE.

Mientras hay un CUE en marcha, marcar una siguiente (doble clic) o un solo
clic en una fila lleva el CUE a esa pista, desde su cue-in; si estaba en
pausa, vuelve a sonar. Una pista cuyo archivo falta o no se puede leer deja
el CUE donde está.
