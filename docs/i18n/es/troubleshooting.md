# Solución de problemas

## Sin sonido {#no-sound}

1. Abre **Configuración → Salidas de audio** y pulsa **Probar Main** para el
   player. Si oyes el tono, revisa el fader de volumen del player.
2. Si no oyes nada, elige otro dispositivo u otro par de canales. Los
   cambios en las salidas se aplican tras un reinicio: pulsa **Reiniciar
   ahora** en Configuración.
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
  [Salida bit-perfect](bit-perfect.md#the-bp-badge): volumen al 100 %, sin
  fundido, nada más en las mismas salidas, un archivo sin pérdida que se ha
  analizado y un dispositivo que funciona a la frecuencia del archivo.
- **Un breve silencio antes de una pista.** El dispositivo bit-perfect se ha
  vuelto a abrir a la frecuencia de muestreo de la pista. Para evitarlo,
  mantén la biblioteca a una sola frecuencia.
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

- Aumenta el **Tamaño de búfer** en Configuración (y pulsa **Reiniciar
  ahora**).
- En Linux, permite la planificación en tiempo real. La aplicación se la
  pide al sistema a través de rtkit (D-Bus). También sirve pertenecer al
  grupo `audio` con un límite `rtprio`.
- Evita las unidades de red para la música que suena al aire.

## «La interfaz ha tenido un error» {#the-interface-hit-an-error}

Se ha capturado un error de dibujo. El audio no se ve afectado. Pulsa
**Reiniciar interfaz**. Por favor, infórmanos con los registros.

## Registros e informes de fallos {#logs-and-crash-reports}

Consulta [Datos y copias de seguridad](data-and-backups.md) para la carpeta
de registros. Hay un archivo de registro por día, y se conservan los últimos
14. Los informes de fallos se guardan como `crash-<time>.txt`. Define
`RUST_LOG=debug` en el entorno para obtener más detalle. Adjunta ambos
archivos al informar de un error.
