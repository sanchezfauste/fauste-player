# Solución de problemas

## Sin sonido {#no-sound}

1. Abre **Configuración → Salidas de audio** y pulsa **Probar Main** para el
   player. Si oyes el tono, revisa el fader de volumen del player.
2. Si no oyes nada, elige otro dispositivo u otro par de canales. Un cambio
   en las salidas se aplica cuando no suena nada en ellas; **Aplicar ahora**
   en Configuración lo aplica al momento.
3. En Linux, es preferible **PipeWire** o **PulseAudio** en Configuración →
   Salidas de audio → Sistema de audio. Comparten la tarjeta de sonido con
   otros programas. **ALSA** habla directamente con la tarjeta y puede
   encontrarla ocupada.
4. **JACK** aparece como no disponible (no hay dispositivo de salida) cuando
   no hay ningún servidor JACK en marcha. Arranca el servidor (por ejemplo
   con QjackCtl) y reinicia la aplicación. Ajusta el servidor JACK a la
   frecuencia de muestreo de Configuración (48 kHz por defecto): JACK
   funciona a una única frecuencia para todos los programas.
5. Los archivos comprimidos que se descargan no ofrecen **PipeWire**; llegan
   a PipeWire a través de su servicio de PulseAudio, que funciona igual.
   Está disponible en las compilaciones hechas con la feature `pipewire`.

## Bit-perfect {#bit-perfect}

- **El distintivo BP sigue apagado.** Comprueba cada condición de
  [Salida bit-perfect](bit-perfect.md#the-bp-badge): sin fundido, nada más
  en las mismas salidas, un archivo sin pérdida que se ha analizado y un
  dispositivo que funciona a la frecuencia del archivo.
- **Un breve silencio antes de una pista.** El dispositivo bit-perfect se ha
  vuelto a abrir a la frecuencia de muestreo de la pista. Solo ocurre cuando
  cambia la frecuencia: las pistas a la misma frecuencia se siguen sin
  hueco. Para evitarlo, mantén la biblioteca a una sola frecuencia. Véase
  [Lo que cuesta](bit-perfect.md#what-it-costs).
- **Las pistas no se mezclan, y Play no hace fundido.** En un player cuya
  salida Main es bit-perfect, cada pista suena hasta el final y luego
  empieza la siguiente, y Play / Siguiente y Anterior cambian al momento: en
  un player así no hay fundidos encadenados ni segues. Desactiva bit-perfect
  en el dispositivo, o mueve el player a otra salida, para volver a mezclar.
  Véase [Lo que cuesta](bit-perfect.md#what-it-costs).
- **El fader de volumen no se mueve.** La salida Main del player es
  bit-perfect, lo que mantiene el volumen al 100 %; la información emergente
  del fader lo dice. Los faders MIDI y la API remota tampoco pueden moverlo.
  Véase [Lo que cuesta](bit-perfect.md#what-it-costs).
- **Una pista suena remuestreada y el registro dice que el dispositivo está
  ocupado (Linux).** Para cambiar de frecuencia, la aplicación cierra el
  dispositivo y lo vuelve a abrir. En ese momento el servidor de sonido
  (PipeWire) puede quedarse con la tarjeta. La aplicación lo vuelve a
  intentar unas cuantas veces; si la tarjeta sigue ocupada, la pista suena a
  la frecuencia actual del dispositivo, y la siguiente pista vuelve a pedir
  su frecuencia. Para darle a la aplicación la tarjeta en exclusiva, abre la
  configuración de sonido del sistema y pon el perfil de esa tarjeta en
  **Apagado** (o **Pro Audio**), para que el servidor de sonido deje en paz
  su dispositivo `hw:`. El número de intentos y la espera entre ellos son
  `tuning.device_busy_retries` y `tuning.device_busy_retry_ms` en el archivo
  de configuración.
- **El dispositivo suena, pero el distintivo BP sigue apagado (Windows o
  macOS).** Se ha denegado el acceso exclusivo, y el dispositivo suena
  compartido.
  - Windows: puede que otro programa tenga el dispositivo en exclusiva, o
    que el control exclusivo esté desactivado en las propiedades Avanzadas
    del dispositivo.
  - macOS: puede que otro programa tenga el dispositivo en modo hog, o que
    el dispositivo solo ofrezca sus frecuencias como un rango continuo (la
    mayoría de interfaces listan frecuencias fijas).
- **No se puede abrir un dispositivo `hw:` (Linux).**
  - Puede que un servidor de sonido tenga la tarjeta. Detenlo, o configura
    el servidor para que deje en paz esa tarjeta, y reinicia la aplicación.
  - Algunos DAC USB solo aceptan muestras de 24 bits empaquetadas
    (`S24_3LE`), que la biblioteca de audio no admite. Usa esa tarjeta a
    través de `plughw:` (no bit-perfect).

### DSD {#dsd}

- **Una pista DSD suena convertida aunque el dispositivo está en DoP o DSD
  nativo.** El registro explica por qué («DSD converted to PCM» y el motivo)
  en estos casos: el volumen del player no está al 100 %, suena algo más en
  el dispositivo, hay más de dos canales, o el dispositivo rechaza la
  frecuencia (DoP necesita la frecuencia DSD dividida entre 16, por ejemplo
  176,4 kHz para DSD64) o no tiene un formato de 24 o 32 bits. Una pista que
  aún no se ha analizado se convierte sin avisar, sin línea en el registro:
  analízala (Configuración → Análisis) y vuelve a reproducirla.
- **Solo la primera pista de un álbum DSD sale como DSD.** Es el ajuste de
  mezcla por defecto: las pistas que el player inicia por sí solo suenan
  convertidas. Elige **Mantener el DSD y silenciar las demás fuentes** en
  Configuración → Salidas de audio para mantenerlas en DSD. Consulta
  [DSD](bit-perfect.md#dsd).
- **La cabecera muestra DSD pero el convertidor reproduce ruido o no se
  engancha.** El convertidor no reconoce DoP (o el formato nativo). Vuelve a
  poner el dispositivo en **Convertir a PCM**.
- **Un chasquido cuando una pista DSD empieza, se detiene o deja el DSD.**
  El convertidor necesita más silencio DSD: sube **Silencio DSD** (200 ms
  por defecto) en Configuración → Salidas de audio, Avanzado.
- **Los demás players o cartuchos no suenan en el dispositivo.** Está
  sonando una pista DSD con **Mantener el DSD y silenciar las demás
  fuentes**; se ve el distintivo **Otras silenciadas**. Vuelven a sonar
  cuando acaba la pista.

## Aviso de «Salida perdida» {#output-lost-alert}

La barra de estado muestra **Salida perdida: &lt;dispositivo&gt;** cuando un
dispositivo deja de responder. Los players siguen contando y mezclando con
un reloj interno, así que la automatización no se detiene. El dispositivo
se vuelve a intentar cada 2 segundos y toma el relevo de nuevo cuando
vuelve. Vuelve a conectar el cable o enciende otra vez la interfaz.

### «Salida perdida» que nunca desaparece, con una salida `hw:` directa {#output-lost-that-never-clears-with-a-direct-hw-output}

Una tarjeta de sonido que se usa a través de una salida ALSA `hw:` directa
(por ejemplo, una salida bit-perfect) la tiene solo Fauste Player: el
servidor de sonido (PipeWire o PulseAudio) no puede usarla a la vez. Si otra
salida pasa por el dispositivo predeterminado del servidor de sonido y ese
dispositivo predeterminado es la misma tarjeta, esa salida nunca arranca y
se queda en **Salida perdida**. El registro dice una vez «output device
opened but never started».

Usa un solo camino por tarjeta: dirige todas las salidas de esa tarjeta a
través del mismo dispositivo `hw:` (con canales distintos si hace falta), o
elige otra tarjeta como salida predeterminada del servidor de sonido en la
configuración de sonido de tu sistema.

## Una pista muestra un icono de aviso o un archivo con una cruz {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

El archivo falta (movido, borrado, desmontado: un archivo con una cruz) o no
se puede decodificar (una señal de aviso). Los players lo saltan. Pasa el
puntero por la fila para ver cuál de los dos es, y la ruta del archivo.

Un archivo que falta se vuelve a buscar cada 30 segundos
(`tuning.missing_recheck_ms` en el archivo de configuración): cuando se
monta la unidad o se vuelve a poner el archivo, la pista se puede reproducir
por sí sola. Un archivo que no se puede decodificar se vuelve a comprobar
por sí solo con el mismo temporizador, por tamaño y fecha de modificación:
no se vuelve a decodificar salvo que cambie uno de los dos, por ejemplo
cuando termina una copia. Para comprobarlo al momento usa **Volver a
analizar** en el menú de su fila, o **Configuración → Análisis → Volver a
analizar todas las pistas** para toda la biblioteca.

Un WAV o FLAC que aparece como ilegible, o que sonaría como un ruido fuerte,
puede ser una extracción de un CD DTS: el archivo contiene un flujo de bits
DTS en lugar de audio. La aplicación lo rechaza (el registro dice «encoded
bitstream stored as PCM») y no decodifica DTS. Conviértelo antes a PCM
normal con otra herramienta.

## Una carpeta vigilada muestra un aviso {#a-watched-folder-shows-a-warning}

Una categoría que vigila una carpeta muestra ⟳, de otro color cuando la
carpeta no se sigue; pasa el puntero por encima para ver el motivo. Véase
[Carpetas vigiladas](library.md#watched-folders).

- **No se encuentra o no se puede leer**: la unidad o el recurso compartido
  no está montado o no se puede alcanzar. Una carpeta vacía que debería
  contener pistas de la biblioteca también cuenta como no encontrada, porque
  un recurso compartido sin montar suele parecer una. No se marca ni se
  quita nada; la carpeta se vuelve a buscar cada 30 segundos y se revisa
  cuando aparece. Si vaciaste la carpeta a propósito, limpia sus pistas
  seleccionándolas en la tabla de la biblioteca y eligiendo **Quitar de la
  biblioteca…**, o eligiendo **Volver a analizar** en ellas, lo que marca
  como faltantes las que ya no tienen archivo para que **Quitar pistas que
  faltan…** pueda quitarlas.
- **Los cambios no se siguen**: el sistema no quiso vigilarla, normalmente
  una carpeta con muchísimas subcarpetas. La carpeta se sigue revisando en
  cada arranque y en cada reintento, así que los archivos nuevos aparecen
  entonces, no al momento. En Linux, un árbol de carpetas mayor que el límite
  de vigilancias por usuario necesita un `fs.inotify.max_user_watches` más
  alto (por ejemplo `sudo sysctl fs.inotify.max_user_watches=524288`; hazlo
  permanente en `/etc/sysctl.d/`).
- **No es una ruta completa**: la carpeta guardada se editó a mano hasta
  dejarla en una ruta que no está completa. Elige **Cambiar carpeta
  vigilada…** para elegirla de nuevo.
- **Un recurso compartido de red no muestra ningún aviso, pero los archivos
  añadidos desde otro ordenador no aparecen**: en un recurso compartido de
  red (SMB/CIFS, NFS) el sistema suele aceptar la vigilancia pero solo
  informa de los cambios hechos desde este ordenador. Los cambios hechos en
  otro sitio se ven en el siguiente arranque, no mientras la aplicación
  funciona.
- **Una carpeta con miles de archivos** tarda un rato en revisarse al
  arrancar; el progreso del análisis lo muestra.
- Las pistas cuyo archivo salió de una carpeta vigilada se quedan en la
  biblioteca marcadas como faltantes. Quítalas con **Quitar pistas que
  faltan…** en el menú de la categoría.

## Cortes de audio {#audio-dropouts}

La barra de estado avisa durante 5 segundos después de cada corte que
detecta la aplicación: **P1: cortes de audio (3)** cuando la decodificación
de un player no ha podido seguir el ritmo del disco (la cuenta es de la
pista que suena ahora), y **&lt;dispositivo&gt;: cortes del dispositivo de
audio (2)** cuando el dispositivo de salida no ha llegado a tiempo (un
xrun). El registro también anota cada uno, como mucho una línea cada 10
segundos por tipo, con cuántos ha habido. No todos los sistemas de audio
informan de los xruns (PulseAudio no lo hace; el modo exclusivo de Windows
tampoco).

- Aumenta el **Tamaño de búfer** en Configuración (se aplica cuando no suena
  nada en ese dispositivo, o al momento con **Aplicar ahora**).
- A través de un servidor de sonido (PulseAudio, el dispositivo `default` de
  ALSA), cada salida ya pide al menos 50 ms sea cual sea el tamaño de
  búfer, para que un ordenador ocupado no se oiga; Configuración lo muestra
  junto a la latencia. En un ordenador antiguo o muy cargado, sube
  `shared_buffer_ms` en la sección `outputs` del archivo de configuración
  (por ejemplo a 100). El sistema mantiene en cola unos dos de esos bloques,
  así que los botones de cartuchos y de play suenan entonces unas dos veces
  esa subida más tarde. Para la salida al aire, la entrada de hardware
  propia del dispositivo (`hw:` en Linux) evita del todo el servidor de
  sonido.
- En Linux, permite la planificación en tiempo real. La aplicación se la
  pide al sistema a través de rtkit (D-Bus), y el registro anota un rechazo.
  Muchos escritorios se la niegan a las aplicaciones.
- Evita las unidades de red para la música que suena al aire.

## La interfaz va lenta en un ordenador antiguo {#the-interface-is-slow-on-an-old-computer}

Mientras algo suena, la interfaz se dibuja como mucho a 30 fotogramas por
segundo (`max_fps` en la sección `ui` del archivo de configuración), y los
clics y las teclas se atienden al momento. Cuando sus fotogramas cuestan más
que una parte del procesador (`frame_budget_percent`, el 40% de un núcleo por
defecto), dibuja menos fotogramas por sí sola, hasta `min_fps` (8 por
defecto), y vuelve a subir cuando salen más baratos. Pasa sobre todo en
ordenadores sin una tarjeta gráfica utilizable, donde el dibujo lo hace el
procesador: el registro lo dice al arrancar ("the interface draws in
software"). Para dejar aún más procesador al audio, baja `max_fps` o
`frame_budget_percent`; 0 en `max_fps` dibuja cada fotograma que muestra la
pantalla. Los vúmetros miden lo mismo a cualquier frecuencia; solo se dibujan
con menos frecuencia. Sin nada sonando, la interfaz se dibuja una vez por
segundo (por el reloj) y cada vez que algo cambia.

## No se pueden soltar archivos en Linux {#files-cannot-be-dropped-on-linux}

La ventana solo acepta archivos arrastrados desde un gestor de archivos
cuando funciona en X11. Por defecto lo hace, también en un escritorio
Wayland (a través de XWayland, que tienen todos los escritorios). Si
`ui.window_system` en `config.json` está en `"Wayland"`, la ventana es una
ventana Wayland nativa, que puede verse más nítida con escalado fraccionario
de la pantalla, pero no se pueden soltar archivos en ella: usa **+ Añadir**
en su lugar. Vuelve a ponerlo en `"Auto"` (el valor por defecto) o `"X11"` y
reinicia Fauste Player.

## Un controlador MIDI no responde {#a-midi-controller-does-not-respond}

Si **Configuración → MIDI** muestra el controlador como conectado pero sus
botones y faders no hacen nada, pulsa allí **Reconectar puertos**: el
programa cierra los puertos MIDI y los vuelve a abrir, como si desenchufaras
y volvieras a enchufar el controlador. Si eso no ayuda, desenchufa el
controlador y vuelve a enchufarlo.

En cualquier caso, el registro dice qué ha pasado. Lista los puertos MIDI
cada vez que cambian, cada puerto que se abre y se cierra (con el motivo),
un puerto que no se puede abrir (una vez, hasta que se abre o desaparece),
el primer mensaje recibido en cada conexión, el primer mensaje ignorado en
cada conexión con el motivo (sin asignación, un fader que aún no ha
alcanzado el volumen, una acción que no puede actuar ahora) y el primer
mensaje de LED que falló. Un controlador conectado que nunca registra un
primer mensaje no envía nada de lo que recibe el programa. Para cada mensaje
ignorado y cada reintento fallido, arranca el programa con
`RUST_LOG=info,fp_control=debug`.

## «La interfaz ha tenido un error» {#the-interface-hit-an-error}

Se ha capturado un error de dibujo. El audio no se ve afectado. Pulsa
**Reiniciar interfaz**. Por favor, infórmanos con los registros.

## Registros e informes de fallos {#logs-and-crash-reports}

Consulta [Datos y copias de seguridad](data-and-backups.md) para la carpeta
de registros. Hay un archivo de registro por día. Las líneas idénticas que se repiten una
tras otra se escriben una sola vez, seguidas de una línea como
`previous message repeated 12 times` (durante una larga serie de repeticiones
esa línea se escribe cada minuto). Las líneas que difieren, por ejemplo
porque llevan un contador, no se agrupan. Los mensajes de conexión
rutinarios del cliente de PulseAudio se omiten por defecto. Define
`RUST_LOG=debug` en el entorno para obtener más detalle (sustituye al filtro
por defecto, `info,pulseaudio=warn`).

El registro tiene un límite: el archivo de un día deja de crecer a los
10 MiB (su última línea lo dice; el resto de ese día no se registra, que es
a lo que lleva una avalancha de líneas distintas) y se conservan los últimos
14 archivos. Cámbialos en el archivo de configuración con
`limits.max_log_file_mb` (de 1 a 1024), `limits.max_log_files` (de 2 a 365)
y `limits.log_repeat_summary_secs` (de 1 a 3600, 60 por defecto, cada cuánto
se informa de una larga serie de repeticiones); se aplican en el siguiente
arranque. Los informes de fallos se guardan como `crash-<time>.txt` en la
misma carpeta y tienen un límite aparte (`limits.max_crash_reports` por
ejecución). Adjunta ambos archivos al informar de un error.
