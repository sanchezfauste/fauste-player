# Configuración

Abre **Configuración** en la barra superior. Ciérrala con **Cerrar** o con
`Esc`. La mayoría de los cambios se aplican al momento y se guardan
automáticamente.

La ventana tiene un único tamaño (900 × 640, más pequeña en una pantalla
pequeña) sea cual sea la sección, y la sección se desplaza dentro de ella.
Cada sección alinea sus etiquetas en una columna.

Las secciones Players, Vúmetros, Análisis, Sonoridad, Automatización y Atajos de teclado tienen un
botón **Restaurar valores por defecto** en su cabecera. Pide confirmación y
luego restablece solo esa sección (Players conserva el número de players y
el idioma; Atajos de teclado no tiene ningún otro botón para restablecer).
Salidas de audio, Playlists, Cartuchera, Biblioteca, MIDI y Remoto no lo tienen.

## Ajustes pendientes {#settings-pending}

Todos los ajustes se aplican mientras la aplicación funciona. El sistema de
audio, la frecuencia de muestreo, el tamaño de búfer (también el propio de
un dispositivo), las salidas Main y Cue (players y cartuchera), los
dispositivos bit-perfect y los ajustes de DSD cambian cómo se abre un
dispositivo de salida, así que se aplican en cuanto es seguro hacerlo:

- Los ajustes de un dispositivo se aplican cuando no suena nada en ese
  dispositivo: ningún player dirigido a él está sonando, haciendo un fundido
  ni preescuchando (CUE, también cuando se mantiene), y ningún cartucho
  suena en él. Un player en pausa, o con una pista solo cargada, no retiene
  un dispositivo: su pista se queda en pausa donde estaba.
- La salida de un player o de la cartuchera se mueve cuando ese player (o la
  cartuchera) no está sonando; los demás players en los mismos dispositivos
  no se interrumpen.
- Un sistema de audio nuevo se aplica cuando no suena nada en ningún sitio.

Un cambio en espera se ejecuta en el primer momento en que no suena nada de
lo que afecta. Puede ser el hueco entre el final de una pista y el comienzo
de la siguiente: la siguiente pista empieza entonces más tarde, por el
tiempo que tarda el dispositivo en volver a abrirse. **Aplicar ahora** (más
abajo) fuerza el cambio al momento.

Una frecuencia o un búfer propios de un dispositivo solo cuentan cuando
cambian aquello con lo que se abre el dispositivo: darle a un dispositivo el
mismo valor que el global, o quitar ese valor, no cambia nada.

Hasta entonces la barra superior muestra **Ajustes pendientes**. Pasa el
puntero por encima para ver qué espera y a qué; haz clic para ver la lista,
que también muestra cualquier cambio que un dispositivo o una salida
rechazó (el dispositivo conserva sus ajustes actuales; el cambio se vuelve a
intentar cuando lo cambias o pulsas **Aplicar ahora**). El pie de
Configuración dice «Algunos cambios esperan a que las salidas que afectan
queden libres.» y ofrece **Aplicar ahora**; un aviso breve (por ejemplo, que
se ha guardado un ajuste) puede ocupar un momento el lugar de ese texto.

**Aplicar ahora** aplica al momento todos los cambios de salida en espera.
Cuando eso interrumpiría algo, primero lista los dispositivos y lo que suena
en ellos; **Interrumpir y aplicar** interrumpe entonces brevemente el audio
en esos dispositivos: lo que sonaba continúa desde donde estaba tras un
hueco breve, y no empieza nada de lo que estaba en pausa o detenido.
**Cancelar** (o `Esc`) sigue esperando.

Los límites y los ajustes del motor se editan en el archivo de configuración
con la aplicación cerrada (consulta [Datos y copias de seguridad](data-and-backups.md)).

## Salidas de audio {#audio-outputs}

Los cambios de esta sección se aplican mientras la aplicación funciona, en
cuanto no suena nada de lo que afectan: consulta [Ajustes
pendientes](#settings-pending).

![Configuración, Salidas de audio, vista Básico: el selector, el sistema de audio, la frecuencia de muestreo, el tamaño de búfer y las salidas Main y Cue de cada player (aquí, el sistema en silencio)](../../images/guide/settings-outputs.png)

Arriba, **Mostrar** elige **Básico** o **Avanzado**. Básico muestra el
sistema de audio, la frecuencia de muestreo, el tamaño de búfer y las
salidas. Avanzado añade, para cada dispositivo que usa una salida, su propia
frecuencia y su propio búfer, el interruptor bit-perfect y el modo DSD, y
después los ajustes de DSD. Cambiar de vista solo muestra u oculta filas: no
se cambia ni se restablece nada. Cuando Básico oculta un ajuste que está en
uso, una línea lo indica. Cuando ninguna salida usa ya un dispositivo, su
frecuencia y su búfer propios, su interruptor bit-perfect y su modo DSD se
olvidan la próxima vez que arranca la aplicación: si después una salida
vuelve a usarlo, empieza con los valores globales. Hasta entonces, volver a
elegirlo (por ejemplo, tras intercambiar dos dispositivos) los conserva.

| Ajuste | Significado |
|---|---|
| Sistema de audio | La última opción, **Sin salida (silencio)**, no reproduce nada: las líneas de tiempo avanzan a ritmo real sin tarjeta de sonido (para una máquina que no la tiene, o para ensayar). Linux: PipeWire (en las compilaciones que lo incluyen), PulseAudio, JACK o ALSA. Windows: WASAPI, ASIO (en las compilaciones que lo incluyen) o JACK. macOS: Core Audio o JACK. Los sistemas que no existen en este ordenador, o que no tienen ningún dispositivo de salida (un servidor JACK que no está en marcha), aparecen como no disponibles. «Predeterminado del sistema» usa el primero disponible en ese orden. |
| Frecuencia de muestreo | La frecuencia a la que funcionan todas las salidas salvo que un dispositivo tenga la suya (Avanzado); los archivos se convierten a ella con un remuestreo de alta calidad. Los dispositivos bit-perfect empiezan a su frecuencia y luego siguen a los archivos. |
| Tamaño de búfer | Fotogramas por bloque de audio, salvo que un dispositivo tenga el suyo; debajo se muestra la latencia resultante. Las salidas a través de un servidor de sonido o un mezclador del sistema (PulseAudio, el dispositivo `default` de ALSA) piden al menos 50 ms sea cual sea este tamaño, o lo que admita el dispositivo, para que un ordenador ocupado no se oiga; la latencia lo indica entonces (`shared_buffer_ms` en el archivo de configuración) |
| Salidas por player | Para cada player, y para la columna **Música** (el [musicalizador](automation.md)), un dispositivo **Main** (al aire) y uno **Cue** (preescucha), cada uno con un par de canales. Una tarjeta de sonido que ofrece varios perfiles de salida (ALSA lista front, surround, hardware directo…) muestra cada uno como *tarjeta — perfil*; dos entradas que seguirían leyéndose igual llevan entre paréntesis el identificador del dispositivo. Las interfaces multicanal pueden llevar varios players en pares distintos. Se permite una salida Cue que comparta una salida en antena (la Main de cualquier player o la de la cartuchera): una nota debajo dice que su preescucha se oye en antena y que pregunta primero si algo suena allí. |
| Probar Main / Probar Cue | Reproduce un tono breve (1 kHz en Main, 440 Hz en Cue, 1,5 s, −18 dBFS) en la salida elegida, para comprobar el cableado antes de salir al aire |
| Biblioteca | La salida Cue de la biblioteca, con su propio **Probar Cue**: donde se oye la [preescucha](library.md#pre-listening) de la ventana de la biblioteca (en un teléfono, la de la pestaña Biblioteca). Sin ella, Preescuchar en la barra de preescucha está atenuado. En Android su opción vacía dice *Predeterminado del sistema* en lugar de *Ninguna*: la preescucha y **Probar Cue** suenan entonces en la salida por defecto (véase [Android → Biblioteca](android.md#library)). Una salida Biblioteca en una salida en antena funciona como la Cue de un player: una nota debajo dice que su preescucha se oye en antena y que pregunta primero si algo suena allí. Está en las vistas Básico y Avanzado, también en un teléfono |
| Cartuchera | Las salidas Main y Cue de la cartuchera. Main es por defecto la salida del sistema. Sin Cue no hay preescucha de cartuchos; una Cue en una salida en antena funciona como en los players. |
| Frecuencia de muestreo: *dispositivo* (Avanzado) | **Global (...)** usa la frecuencia de muestreo de arriba; un valor le da a este dispositivo su propia frecuencia. Solo se ofrecen las frecuencias que indica el dispositivo; una frecuencia guardada que ya no indica sigue en la lista con una nota de que puede que no se abra (entonces el dispositivo vuelve a la frecuencia global). Los valores propios solo se aplican a dispositivos que nombra una salida, no a la salida predeterminada del sistema salvo que una la nombre. |
| Tamaño del búfer: *dispositivo* (Avanzado) | **Global (...)** usa el tamaño de búfer de arriba (con el mínimo que pide una salida compartida, como arriba); un valor le da a este dispositivo el suyo, usado tal cual, con su latencia debajo. Un dispositivo que no acepta su propio tamaño de búfer vuelve al global, y también a la frecuencia global si tampoco acepta su propia frecuencia. |
| Bit perfect: *dispositivo* (Avanzado) | Desactivado por defecto. Un modo especial para la reproducción audiófila, no para programas con mezclas: el dispositivo se abre con acceso exclusivo y sigue la frecuencia de muestreo de cada pista. Los players cuya salida Main está en él nunca mezclan ni encadenan con fundido, un cambio de frecuencia deja un silencio breve entre dos pistas, y su volumen se mantiene al 100 %. El interruptor está desactivado donde el dispositivo no puede dar acceso exclusivo. Consulta [Lo que cuesta](bit-perfect.md#what-it-costs). |
| DSD: *dispositivo* (Avanzado) | **Convertir a PCM** (por defecto), **DoP** o, en Linux, **DSD nativo**. Todos los dispositivos lo muestran; solo se ofrecen los modos que el dispositivo admite, y una línea debajo explica por qué no los demás. Consulta [DSD](bit-perfect.md#dsd). |
| Cuando otra fuente necesita una salida DSD (Avanzado) | **Seguir la pista DSD en PCM** (por defecto) o **Mantener el DSD y silenciar las demás fuentes**. Consulta [DSD](bit-perfect.md#dsd). |
| Silencio DSD (Avanzado) | Silencio que se envía antes de que empiece un flujo DSD, al acabar y al pasar a PCM, para que el convertidor se enganche sin chasquido; 200 ms por defecto, de 0 a 2000. |

Una salida Cue nunca recurre a la salida que usa Main. Una Cue que nombra un
dispositivo de un sistema de audio que este ordenador no tiene significa
«sin cue». Una Cue en una salida en antena (el mismo dispositivo y los
mismos canales que una Main, o solapándose con ellos) se permite: una nota
debajo dice que su preescucha se oye en antena y que pregunta primero si
algo suena allí. Un player sin salida Cue tiene el botón **CUE** atenuado;
al pasar el puntero por encima te indica que elijas aquí una salida Cue. Lo
mismo vale para **Preescuchar en CUE** de la cartuchera.

Si un dispositivo desaparece mientras suena, los players mantienen sus
líneas de tiempo, y el dispositivo se vuelve a abrir cuando regresa
(consulta [Solución de problemas](troubleshooting.md)).

## Players {#players}

![Configuración, Players: número de players, modo por defecto, duración del fundido entre pistas, duración del Stop con fundido, mezcla automática, cue-in y cue-out, aviso de fin de pista e idioma](../../images/guide/settings-players.png)

| Ajuste | Por defecto | Significado |
|---|---|---|
| Idioma | Sistema | Idioma de la interfaz |
| Número de players | 4 | Columnas de la pantalla principal (no se puede quitar un player que está sonando). El número se aplica al soltar el deslizador; un número escrito en la casilla se aplica al pulsar `Enter` o al salir de la casilla, y `Esc` lo cancela |
| Modo por defecto | CONT | El modo con el que arrancan los players |
| Duración del fundido entre pistas | 1000 ms | Cuánto tarda en bajar la pista que suena cuando pulsas Play (siguiente) mientras suena, y con Pista anterior. No se aplica a un player cuya salida Main es bit-perfect |
| Duración del Stop con fundido | 2000 ms | Cuánto tarda **Stop con fundido** en bajar la pista. Se aplica a todos los players, también a los bit-perfect |
| Mezcla automática en el punto MIX | Activada | Solapa las pistas en modo continuo. No se aplica a un player cuya salida Main es bit-perfect |
| Usar cue-in y cue-out | Activado | Desactivado: los players reproducen cada pista desde el principio hasta el final del archivo; los marcadores se conservan y los cartuchos siguen usando los suyos. Las duraciones y los totales de las playlists siguen el mismo rango |
| Aviso de fin de pista | 10 s | Cuándo empieza a parpadear en rojo la cuenta atrás |

Mientras la salida Main de un player es bit-perfect, una nota bajo **Mezcla
automática en el punto MIX** dice que esos players nunca mezclan ni
encadenan con fundido, así que la duración del fundido entre pistas y la
mezcla automática no se les aplican. Ambos ajustes siguen siendo editables: se siguen
aplicando a los demás players. Véase [Lo que cuesta](bit-perfect.md#what-it-costs).

## Vúmetros {#meters}

![Configuración, Vúmetros, con el medidor de pico digital elegido](../../images/guide/settings-meters.png)

Los cambios se aplican al momento. Configuración muestra solo lo que usa el
tipo de medidor elegido: los medidores EBU, DIN y VU tienen la escala, la
zona roja y el comportamiento que fija su norma (solo se ajusta el nivel de
alineación), y la alineación de un medidor K-System es su propio 0. Un valor
que fijes se conserva para cuando vuelvas a elegir ese tipo.

| Ajuste | Por defecto | Significado |
|---|---|---|
| Tipo de medidor | Pico digital | Cómo sube y baja la barra, y su escala, según una norma (ver más abajo) |
| Tiempo de subida, Velocidad de caída | 5 ms, 11,8 dB/s | Solo para **Personalizado**. El tiempo de subida es un tiempo de integración: una ráfaga de tono de esa duración marca 2 dB menos; 0 muestra todos los picos. |
| True peak | Desactivado | Solo pico digital, personalizado y K-System. Mide entre muestras, con el filtro de sobremuestreo 4× que publica ITU-R BS.1770. Muestra picos que superan 0 dBFS tras la conversión y que un medidor de pico de muestra no ve. Como permite la norma, un clic aislado de una sola muestra puede marcar hasta unos 0,3 dB por debajo del valor de su muestra. |
| Suelo de la escala | −60 dBFS | La parte inferior de la escala digital (pico digital y personalizado). Los demás medidores muestran el rango que da su norma. |
| Retención del pico | 2 s | Solo pico digital, personalizado y K-System: cuánto tiempo se mantiene encendido el nivel más alto; 0 la desactiva. Los medidores de programa y el VU no tienen retención. |
| Nivel de alineación | −18 dBFS | Todos menos el K-System. Se marca en la escala (EBU R68). También es donde se sitúan la marca TEST del EBU, la marca −9 del DIN y 0 VU. |
| Aviso desde | −9 dBFS | Amarillo desde aquí (máximo permitido EBU), para los medidores de pico digital y personalizado |
| Peligro desde | −3 dBFS | Rojo desde aquí, para los medidores de pico digital y personalizado. Los demás se ponen rojos donde lo hace su escala: el VU desde 0 VU, los PPM EBU y DIN desde el máximo permitido (EBU +9, DIN 0) y el K-System desde +4. |
| Lectura de sonoridad | Corto plazo | La sonoridad bajo el vúmetro: desactivada, momentánea (últimos 400 ms) o a corto plazo (últimos 3 s), EBU R128 |
| Objetivo de sonoridad | −23 LUFS | La lectura se ve en verde a ±1 LU (EBU R128) |

| Tipo de medidor | Norma | Comportamiento |
|---|---|---|
| Pico digital | IEC 60268-18 | Muestra cada pico al instante; cae 20 dB en 1,7 s |
| PPM EBU | IEC 60268-10 tipo IIb | Los picos de menos de unos 10 ms marcan menos (una ráfaga de tono de 10 ms marca unos 1,6 dB menos, una de 0,5 ms unos 18 dB menos), dentro de las tolerancias de EBU Tech 3205; cae 24 dB en 2,8 s |
| PPM DIN | IEC 60268-10 tipo I | Lo mismo con un tiempo de integración de 5 ms; cae 20 dB en 1,5 s |
| VU | IEC 60268-17 | El nivel medio, con el movimiento de aguja de un vúmetro VU: 99 % en 300 ms, con un ligero rebase; una senoide marca su nivel de pico |
| K-20, K-14, K-12 | K-System | Dos secciones: la media (RMS, 600 ms) como barra sólida y el pico (cae 26 dB en 3 s) atenuado por encima. 0 está 20, 14 o 12 dB por debajo de la escala completa; verde por debajo de 0, ámbar de 0 a +4 y rojo por encima. K-12 va bien para emisión; K-14 y K-20, para programas más dinámicos. |
| Personalizado | — | Tu tiempo de subida y tu velocidad de caída |

Cada medidor usa la escala de su norma, con sus marcas entre los canales:

| Medidor | Escala |
|---|---|
| Pico digital, personalizado | −60 … 0 dBFS, marcas cada 10 dB hasta −40 y cada 5 dB por encima; los 20 dB superiores ocupan la mitad de la altura |
| PPM EBU | −12 … +12 alrededor del nivel de alineación (TEST), cada 4 dB; los niveles más bajos se quedan abajo del todo |
| PPM DIN | −50 … +5, donde 0 está 9 dB por encima del nivel de alineación (−9 dBFS por defecto) |
| VU | −20 … +3 VU, 0 VU en el nivel de alineación; la barra se mueve en proporción a la tensión, como la aguja |
| K-System | desde +20, +14 o +12 (0 dBFS) hasta −60; uniforme en dB hasta −24 |

## Análisis {#analysis}

![Configuración, Análisis: los umbrales de los marcadores automáticos](../../images/guide/settings-analysis.png)

Los umbrales que se describen en [Marcadores y mezcla](markers-and-mixing.md);
cada fila tiene bajo su nombre una indicación de una línea que dice qué hace
el valor. **Volver a analizar todas las pistas** repite el análisis de toda
la biblioteca, leyendo otra vez cada archivo; los marcadores manuales se
conservan. Mientras se ejecuta, el botón aparece atenuado y muestra cuánto
lleva (**Analizando… 120 de 340**), de modo que pulsarlo otra vez no empieza
de nuevo; la información emergente de **Analizando N pistas** de la barra de
estado da el mismo progreso. Cambiar aquí un umbral también vuelve a analizar
la biblioteca, y reinicia una ejecución que ya está en marcha para que
ninguna pista conserve marcadores de los valores antiguos.

Tras una actualización que ha cambiado el análisis, las pistas analizadas
por la versión anterior conservan sus marcadores y formas de onda, que
siguen funcionando. Al arrancar, Fauste Player dice cuántas hay y ofrece
**Analizar ahora** o **Más tarde**; **Analizar las pistas desactualizadas
(N)**, aquí, hace lo mismo en cualquier momento. Las pistas de los players
se ponen al día igualmente, a medida que se muestran, igual que las pistas
de los cartuchos que no tienen un formato registrado (un cartucho solo suena
bit-perfect cuando se conoce su formato). Las pistas cuyo archivo falta no
se cuentan hasta que el archivo vuelve.

El análisis también mide la sonoridad de cada archivo para la
[normalización de la sonoridad](loudness.md). Los archivos analizados antes
de que se midiera la sonoridad se miden en segundo plano, cuando el análisis
no tiene nada más que hacer, esté la normalización activada o no; sus
marcadores no se tocan. **Volver a analizar** (en el menú de la playlist) y
**Volver a analizar todas las pistas** miden de nuevo.

**Buscar archivos cambiados** revisa todas las pistas de la biblioteca en
busca de un archivo que otro programa haya cambiado mientras Fauste Player
funcionaba, y vuelve a analizar los cambiados; un aviso dice cuántos ha
encontrado. El botón aparece atenuado mientras dura una búsqueda. Véase
[Cuando un archivo cambia](library.md#when-a-file-changes).

## Sonoridad {#loudness}

![Configuración, Sonoridad: normalización activada, con el objetivo, el techo de true peak, la ganancia máxima y el ajuste manual máximo en sus valores por defecto](../../images/guide/settings-loudness.png)

Reproduce todas las pistas y todos los cartuchos con la misma sonoridad,
con una ganancia fija por pista. La [normalización de la
sonoridad](loudness.md) explica cómo se calcula la ganancia, el distintivo
de la cabecera del player y los ajustes por pista.

| Ajuste | Por defecto | Rango | Significado |
|---|---|---|---|
| Normalizar la sonoridad | Activada | | Normalización activada o desactivada |
| Objetivo | −16 LUFS | −36 … −6 LUFS, de 0,5 en 0,5 | La sonoridad integrada a la que se lleva cada pista |
| Techo de true peak | −1 dBTP | −12 … 0 dBTP, de 0,1 en 0,1 | Una subida nunca lleva el true peak de la pista por encima de este valor. Las bajadas no se limitan |
| Ganancia máxima | 12 dB | 0 … 24 dB | La mayor ganancia que puede recibir una pista baja. Las bajadas no se limitan |
| Ajuste manual máximo | 12 dB | 0 … 24 dB | Hasta dónde puede llegar el ajuste manual propio de cada pista, hacia arriba o hacia abajo |

Los cambios se aplican a las pistas que se carguen después: lo que suena
conserva su ganancia, y la siguiente pista que espera en cada player se
vuelve a preparar con la nueva. Una nota bajo los ajustes dice cuántas
pistas siguen esperando su medición; suenan sin cambios hasta que termina.
Un player o un cartucho en una salida bit-perfect nunca se normaliza.
**Restaurar valores por defecto** restablece solo esta sección
(normalización activada). La normalización está activada en las
instalaciones nuevas; una configuración existente conserva su ajuste actual
(toda configuración guardada ya almacena el valor).

## Playlists {#playlists}

![Configuración, Playlists: la carpeta de música, las playlists y las columnas de la tabla](../../images/guide/settings-playlists.png)

- **Carpeta de música:** donde empiezan los diálogos de archivos.
- **Nueva playlist**, **Renombrar** (edita el nombre y pulsa Enter; Esc
  cancela) y **Eliminar** (icono de la papelera).
- **Importar M3U / PLS…** crea una playlist nueva a partir de un archivo de
  playlist. **M3U**, en cada fila, la exporta como M3U8. Consulta
  [Playlists](playlists.md).
- **Columnas de la tabla:** qué columnas muestran las tablas de pistas y en
  qué orden, para todos los players: una casilla por columna (Título y Dur.
  no se pueden desactivar), flechas arriba y abajo para las visibles, y
  **Columnas por defecto**. Consulta [Playlists](playlists.md).

**Idioma:** una lista desplegable: **Sistema** (sigue al sistema operativo)
y luego cada idioma en que está disponible la interfaz, cada uno con su
propio nombre (primero English y después por orden alfabético: por ejemplo,
Español). La interfaz cambia al momento. Un idioma del sistema sin
traducción propia usa el más cercano (el francés de Canadá usa el francés;
el portugués de Brasil, el portugués), y si no, el inglés. Un idioma del
archivo de configuración que la interfaz no tiene aparece como **Sistema** y
sigue al sistema operativo.

La interfaz en todos los idiomas, incluidos el inglés y el español, se ha
generado con IA y puede contener errores. **Acerca de** lleva un aviso legal
general que los cubre a todos. Las correcciones son bienvenidas como issues
o pull requests.

## Cartuchera {#cartwall}

![Configuración, Cartuchera: las páginas, la rejilla y el editor del cartucho seleccionado](../../images/guide/settings-cartwall.png)

Páginas, tamaño de la rejilla, el editor de cartuchos y la importación y
exportación de páginas de cartuchos. Consulta [Cartuchera](cartwall.md).

## Biblioteca {#library}

Crea, renombra, cambia el color y el tipo, mueve (arrastrando) y elimina las
categorías que agrupan tus pistas, vigila una carpeta con una categoría y
quita sus pistas que faltan, sin abrir la ventana de la biblioteca. Consulta
[Biblioteca](library.md#categories) y
[Carpetas vigiladas](library.md#watched-folders).

**Editor externo** es el editor de audio que *Abrir en el editor externo*
abre con el archivo de una pista: la ruta del programa (o *Ninguno*),
**Elegir…** para escogerlo y **Quitar** para borrarlo. En macOS se puede
elegir un paquete de aplicación (`.app`). Si el programa no se puede
iniciar, un aviso lo dice. La fila no se muestra en Android, ni en las
versiones Flatpak y snap. Véase [Cuando un archivo
cambia](library.md#when-a-file-changes).

## Automatización {#automation}

Las rotaciones y las reglas del [musicalizador](automation.md) (no en
Android).

![Configuración, Automatización: las rotaciones Música (la por defecto) y Hits, la Música seleccionada con su único paso sobre toda la música, y las reglas del musicalizador](../../images/guide/settings-automation.png)

- **Rotaciones:** la lista, con la por defecto marcada. **Nueva rotación**
  añade una que reproduce toda la música, **Hacer por defecto** convierte la
  seleccionada en la por defecto y **Borrar rotación** la elimina (la última
  no se puede borrar).
- **La rotación seleccionada:** su **Nombre**, y luego una fila por cada
  **paso**: de dónde salen sus pistas (una categoría con sus subcategorías,
  o *Todo* de un tipo), cuántas reproduce cada vez (**Como mínimo** 0–99,
  **Como máximo** 1–99; un mínimo de 0 permite que el paso se salte algunos
  turnos), **Subir**, **Bajar** y **Quitar**. **Añadir paso** añade uno al
  final. Un paso cuya categoría se borró lo dice; el musicalizador se lo
  salta.
- **Musicalizador:** cuánto tiempo debe pasar antes de que la misma pista
  pueda volver a sonar (3 horas, o cuando haya sonado la mitad de las demás
  pistas de su paso, lo que ocurra primero; 0 desactiva cualquiera de las
  dos), el mismo artista (30 minutos), el **margen por delante** (30 minutos
  de música en cola) y las **pistas sonadas que se conservan** en su lista
  (50).

**Restaurar valores por defecto** restablece las reglas; las rotaciones se
quedan.

## Atajos de teclado {#keyboard-shortcuts}

![Configuración, Atajos de teclado: cada acción de los players con su tecla, y Quitar junto a las asignadas](../../images/guide/settings-shortcuts.png)

Consulta [Teclado](keyboard.md).

## MIDI {#midi}

![Configuración, MIDI, con el control MIDI desactivado](../../images/guide/settings-midi.png)

Activa las superficies de control MIDI, consulta los puertos de entrada
(**Reconectar puertos** los vuelve a abrir, como si volvieras a enchufar los
controladores) y enseña un control para cada acción de los players. Consulta
[Superficies de control MIDI](midi.md).

## Remoto {#remote}

![Configuración, Remoto, con la API HTTP escuchando en este ordenador](../../images/guide/settings-remote.png)

Control remoto por la red, para páginas web, aplicaciones de móvil,
automatización y superficies de control. Consulta
[Control remoto](remote-control.md).

- **Permitir el control remoto por HTTP**, su **Dirección** y su **Puerto**,
  y una línea que dice si está escuchando.
- **Token**, obligatorio fuera de este ordenador. **Generar** crea uno al
  azar, **Mostrar** lo revela y **Copiar** lo pone en el portapapeles.
  Aparece un aviso cuando la dirección llega más allá de este ordenador y no
  hay token.
- **Páginas web que pueden usar la API**: un origen por línea.
- **Permitir el control por OSC**, su **Dirección** y su **Puerto**, y los
  **Emisores permitidos** (direcciones o subredes, una por línea).
- **Publicar los tiempos cada**: cada cuánto se envían los tiempos
  transcurrido y restante mientras algo suena.

Los campos de texto y los números se aplican al salir de ellos, lo que
incluye abrir otra sección o cerrar Configuración; Esc cancela lo que
estabas escribiendo. Un valor no válido se corrige, y el campo muestra el
valor que se ha conservado.
