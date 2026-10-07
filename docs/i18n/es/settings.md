# Configuración

Abre **Configuración** en la barra superior. Ciérrala con **Cerrar** o con
`Esc`. La mayoría de los cambios se aplican al momento y se guardan
automáticamente.

La ventana tiene un único tamaño (900 × 640, más pequeña en una pantalla
pequeña) sea cual sea la sección, y la sección se desplaza dentro de ella.
Cada sección alinea sus etiquetas en una columna.

Las secciones Players, Vúmetros, Análisis y Atajos de teclado tienen un
botón **Restaurar valores por defecto** en su cabecera. Pide confirmación y
luego restablece solo esa sección (Players conserva el número de players y
el idioma; Atajos de teclado no tiene ningún otro botón para restablecer).
Salidas de audio, Playlists, Cartuchera, MIDI y Remoto no lo tienen.

## Reinicio pendiente {#restart-pending}

Algunos cambios solo surten efecto cuando la aplicación vuelve a arrancar:
el sistema de audio, la frecuencia de muestreo, el tamaño de búfer (también
el propio de un dispositivo), las salidas Main y Cue (players y cartuchera),
los dispositivos bit-perfect y los ajustes de DSD. El número de players no
es uno de ellos: se aplica al momento.

Una frecuencia o un búfer propios de un dispositivo solo cuentan cuando
cambian aquello con lo que se abre el dispositivo: darle a un dispositivo el
mismo valor que el global, o quitar ese valor, no queda pendiente.

Los límites y los ajustes del motor también se aplican en el siguiente
arranque, pero se editan en el archivo de configuración con la aplicación
cerrada (consulta [Datos y copias de seguridad](data-and-backups.md)), así
que nunca aparecen como pendientes.

Mientras alguno de ellos espera, el pie de Configuración dice «Algunos
cambios se aplican tras reiniciar.» y ofrece **Reiniciar ahora**, y la barra
superior muestra la etiqueta **Reinicio pendiente**. Pasa el puntero por la
etiqueta para ver qué está esperando. Un aviso breve (por ejemplo, que se ha
guardado un ajuste) puede ocupar un momento el lugar del texto del pie;
**Reiniciar ahora** se mantiene. Los dos hacen lo mismo:

- Si no hay nada al aire, **Reiniciar ahora** (o la etiqueta) reinicia al
  momento.
- Si hay algo al aire, aparece la ventana que lista lo que está sonando, con
  **Detener y reiniciar** o **Cancelar**.

Primero se guarda la sesión y se detienen el audio y el control MIDI; luego
la aplicación vuelve a arrancar con la misma carpeta de datos
(`FAUSTE_HOME`), y después no sale nada al aire por sí solo. Si la
aplicación no puede volver a arrancar (en un Flatpak, también cuando la
nueva no arranca a tiempo), lo indica; ábrela desde el menú de
aplicaciones.

## Salidas de audio {#audio-outputs}

Los cambios de esta sección esperan a un reinicio: consulta
[Reinicio pendiente](#restart-pending).

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
| Tamaño de búfer | Fotogramas por bloque de audio, salvo que un dispositivo tenga el suyo; debajo se muestra la latencia resultante |
| Salidas por player | Para cada player, un dispositivo **Main** (al aire) y uno **Cue** (preescucha), cada uno con un par de canales. Una tarjeta de sonido que ofrece varios perfiles de salida (ALSA lista front, surround, hardware directo…) muestra cada uno como *tarjeta — perfil*; dos entradas que seguirían leyéndose igual llevan entre paréntesis el identificador del dispositivo. Las interfaces multicanal pueden llevar varios players en pares distintos. |
| Probar Main / Probar Cue | Reproduce un tono breve (1 kHz en Main, 440 Hz en Cue, 1,5 s, −18 dBFS) en la salida elegida, para comprobar el cableado antes de salir al aire |
| Cartuchera | Las salidas Main y Cue de la cartuchera. Main es por defecto la salida del sistema. Sin Cue no hay preescucha de cartuchos. |
| Frecuencia de muestreo: *dispositivo* (Avanzado) | **Global (...)** usa la frecuencia de muestreo de arriba; un valor le da a este dispositivo su propia frecuencia. Solo se ofrecen las frecuencias que indica el dispositivo; una frecuencia guardada que ya no indica sigue en la lista con una nota de que puede que no se abra (entonces el dispositivo vuelve a la frecuencia global). Los valores propios solo se aplican a dispositivos que nombra una salida, no a la salida predeterminada del sistema salvo que una la nombre. |
| Tamaño del búfer: *dispositivo* (Avanzado) | **Global (...)** usa el tamaño de búfer de arriba; un valor le da a este dispositivo el suyo, con su latencia debajo. Un dispositivo que no acepta su propio tamaño de búfer vuelve al global, y también a la frecuencia global si tampoco acepta su propia frecuencia. |
| Bit perfect: *dispositivo* (Avanzado) | Un dispositivo bit-perfect se abre con acceso exclusivo y sigue la frecuencia de muestreo de cada archivo mientras no suena nada en él. El interruptor está desactivado donde el dispositivo no puede dar acceso exclusivo. Consulta [Salida bit-perfect](bit-perfect.md). |
| DSD: *dispositivo* (Avanzado) | **Convertir a PCM** (por defecto), **DoP** o, en Linux, **DSD nativo**. Todos los dispositivos lo muestran; solo se ofrecen los modos que el dispositivo admite, y una línea debajo explica por qué no los demás. Consulta [DSD](bit-perfect.md#dsd). |
| Cuando otra fuente necesita una salida DSD (Avanzado) | **Seguir la pista DSD en PCM** (por defecto) o **Mantener el DSD y silenciar las demás fuentes**. Consulta [DSD](bit-perfect.md#dsd). |
| Silencio DSD (Avanzado) | Silencio que se envía antes de que empiece un flujo DSD, al acabar y al pasar a PCM, para que el convertidor se enganche sin chasquido; 200 ms por defecto, de 0 a 2000. |

Una salida Cue nunca recurre a la salida que usa Main, para que la
preescucha nunca salga al aire. Una Cue que nombra un dispositivo de un
sistema de audio que este ordenador no tiene, o la misma salida (dispositivo
y canales) que Main, significa «sin cue». Cuando una salida Cue es la misma
que su salida Main, un aviso debajo lo indica. Un player sin salida Cue, o
con su Cue en su salida Main, tiene el botón **CUE** atenuado; al pasar el
puntero por encima te indica que elijas aquí una salida Cue. Lo mismo vale
para **Preescuchar en CUE** de la cartuchera.

Si un dispositivo desaparece mientras suena, los players mantienen sus
líneas de tiempo, y el dispositivo se vuelve a abrir cuando regresa
(consulta [Solución de problemas](troubleshooting.md)).

## Players {#players}

![Configuración, Players: número de players, modo por defecto, duración del fundido, mezcla automática, cue-in y cue-out, aviso de fin de pista e idioma](../../images/guide/settings-players.png)

| Ajuste | Por defecto | Significado |
|---|---|---|
| Idioma | Sistema | Idioma de la interfaz |
| Número de players | 4 | Columnas de la pantalla principal (no se puede quitar un player que está sonando) |
| Modo por defecto | CONT | El modo con el que arrancan los players |
| Duración del fundido | 1000 ms | La usan Play mientras suena una pista y Stop con fundido |
| Mezcla automática en el punto MIX | Activada | Solapa las pistas en modo continuo |
| Usar cue-in y cue-out | Activado | Desactivado: los players reproducen cada pista desde el principio hasta el final del archivo; los marcadores se conservan y los cartuchos siguen usando los suyos. Las duraciones y los totales de las playlists siguen el mismo rango |
| Aviso de fin de pista | 10 s | Cuándo empieza a parpadear en rojo la cuenta atrás |

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

Los umbrales que se describen en [Marcadores y mezcla](markers-and-mixing.md).
**Volver a analizar todas las pistas** repite el análisis de toda la
biblioteca; los marcadores manuales se conservan.

Tras una actualización que ha cambiado el análisis, las pistas analizadas
por la versión anterior conservan sus marcadores y formas de onda, que
siguen funcionando. Al arrancar, Fauste Player dice cuántas hay y ofrece
**Analizar ahora** o **Más tarde**; **Analizar las pistas desactualizadas
(N)**, aquí, hace lo mismo en cualquier momento. Las pistas de los players
se ponen al día igualmente, a medida que se muestran, igual que las pistas
de los cartuchos que no tienen un formato registrado (un cartucho solo suena
bit-perfect cuando se conoce su formato). Las pistas cuyo archivo falta no
se cuentan hasta que el archivo vuelve.

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

El inglés y el español están escritos a mano. Las demás traducciones se han
generado con IA y pueden contener errores; cuando una de ellas está en uso,
**Acerca de** lo indica. Las correcciones de hablantes nativos son
bienvenidas como issues o pull requests.

## Cartuchera {#cartwall}

![Configuración, Cartuchera: las páginas, la rejilla y el editor del cartucho seleccionado](../../images/guide/settings-cartwall.png)

Páginas, tamaño de la rejilla, el editor de cartuchos y la importación y
exportación de páginas de cartuchos. Consulta [Cartuchera](cartwall.md).

## Atajos de teclado {#keyboard-shortcuts}

![Configuración, Atajos de teclado: cada acción de los players con su tecla, y Quitar junto a las asignadas](../../images/guide/settings-shortcuts.png)

Consulta [Teclado](keyboard.md).

## MIDI {#midi}

![Configuración, MIDI, con el control MIDI desactivado](../../images/guide/settings-midi.png)

Activa las superficies de control MIDI, consulta los puertos de entrada y
enseña un control para cada acción de los players. Consulta
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
