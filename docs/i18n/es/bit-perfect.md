# Salida bit-perfect

Un dispositivo **bit-perfect** recibe las muestras de cada archivo
exactamente como están en el archivo: la misma frecuencia de muestreo, los
mismos valores, sin remuestreo, cambio de volumen ni mezcla.

La salida bit-perfect es un **modo especial para la reproducción audiófila**:
escuchar un archivo, alimentar una cadena hi-fi o de monitorización,
comprobar un enlace digital, en cualquier caso donde convenga evitar
cualquier procesamiento en el ordenador. Está desactivada por defecto y no
está pensada para un programa musical con mezclas. Lee [Lo que
cuesta](#what-it-costs) antes de activarla.

## Lo que cuesta {#what-it-costs}

Para mantener cada pista intacta, un player cuya salida Main está en un
dispositivo bit-perfect se comporta de forma distinta a los demás:

- **Sin fundidos encadenados ni segues.** Cada pista se reproduce hasta el
  final (su cue-out, o el final del archivo con **Usar cue-in y cue-out**
  desactivado), y luego empieza la siguiente. **Mezcla automática en el
  punto MIX** y el punto MIX no se usan: el marcador MIX se dibuja
  atenuado y el distintivo **Mezcla** no se enciende nunca. **Play /
  Siguiente** y **Anterior** mientras suena una pista cambian al momento,
  con una parada breve sin chasquido, en lugar de hacer un fundido.
- **Un silencio breve cuando cambia la frecuencia de muestreo.** Entre dos
  pistas cuyas frecuencias de muestreo difieren, cuando no suena nada más en
  el dispositivo, este reproduce primero lo que aún tiene en el búfer (el
  final de la pista), y luego se vuelve a abrir a la nueva frecuencia, lo que
  deja un silencio breve (normalmente unas decenas de milisegundos, más en
  algunos dispositivos). Las pistas a la misma frecuencia se siguen sin hueco
  y sin solape. Cuando la frecuencia no puede seguir (véase [Qué ocurre en
  un dispositivo bit-perfect](#what-happens-on-a-bit-perfect-device)), la
  siguiente pista empieza al momento, convertida a la frecuencia del
  dispositivo.
- **Sin normalización de la sonoridad.** Una pista en un dispositivo
  bit-perfect suena siempre a ganancia unitaria, incluso con la
  [normalización de la sonoridad](loudness.md#bit-perfect-outputs) activada:
  la cabecera muestra **0.0 dB** en ámbar, y su información emergente dice
  por qué. Los demás players y cartuchos se normalizan como de costumbre.
- **El volumen se mantiene al 100 %.** El fader de volumen del player está
  bloqueado arriba: en pantalla, en los controladores [MIDI](midi.md) y por
  la [API remota y OSC](remote-control.md). Pasa al 100 % cuando se activa
  bit-perfect en el dispositivo, y se queda ahí cuando se desactiva.
- **Lo que sigue funcionando:** **Stop con fundido** (baja con un fundido
  durante la duración del Stop con fundido), **Pausa**, **Stop**, **Reiniciar**, las
  marcas de repetir y de parar después, los modos Single y Continuo, Cue
  (la preescucha, que no tiene fader y no se ve afectada), los cartuchos, la
  [salida DSD](#dsd), y todos los players cuya salida Main está en otro
  dispositivo, que mezclan como siempre. Un cartucho puede sonar en el
  mismo dispositivo: mientras suena, la siguiente pista del player empieza a
  la frecuencia actual del dispositivo, convertida (así que no es
  bit-perfect), y no se inserta silencio.
- **Cuándo usarla.** Actívala cuando cada pista deba llegar al convertidor
  intacta. Déjala desactivada para un programa en antena que necesite
  transiciones mezcladas o cambios de nivel. El modo se fija por
  dispositivo, así que los players con mezclas pueden quedarse en otro
  dispositivo.

## Hacer bit-perfect un dispositivo {#setting-a-device-bit-perfect}

1. En **Configuración → Salidas de audio**, elige el dispositivo de forma
   explícita para la salida Main de un player (o de la cartuchera). Un
   player que se deja en el predeterminado del sistema no se puede hacer
   bit-perfect. Un dispositivo que ya no usa ninguna salida pierde su
   interruptor bit-perfect y su modo DSD la próxima vez que arranca la
   aplicación.
2. Elige **Avanzado** arriba de la sección. En **Ajustes por dispositivo**,
   activa **Bit perfect** junto al dispositivo.
3. El dispositivo pasa a ser bit-perfect en cuanto no suena nada en él
   (véase [Ajustes pendientes](settings.md#settings-pending)).

A partir de entonces, el dispositivo arranca a su propia frecuencia de
muestreo si se la has dado en ese mismo lugar, o si no a la frecuencia de
muestreo global, y a partir de ahí sigue a cada archivo.

El interruptor está desactivado cuando el dispositivo no puede dar acceso
exclusivo.
- **Linux:** elige un dispositivo ALSA cuyo nombre empiece por `hw:`. Es la
  propia tarjeta de sonido. PulseAudio, PipeWire, JACK y los dispositivos
  ALSA `default` o `plughw:` mezclan o convierten, así que nunca son
  bit-perfect.
- **Windows:** elige el dispositivo en el sistema **WASAPI**. Se abre en
  modo exclusivo.
  - En la configuración de sonido de Windows, las propiedades
    **Avanzadas** del dispositivo deben tener activada *Permitir que las
    aplicaciones tomen el control exclusivo de este dispositivo* (lo está
    por defecto).
  - Mientras suena, ningún otro programa puede usar el dispositivo.
- **macOS:** elige el dispositivo en **Core Audio**. Se abre en modo hog
  (exclusivo).
  - La frecuencia de muestreo del dispositivo se ajusta a la de la pista, y
    su formato al formato entero más amplio que ofrece a esa frecuencia (los
    ajustes que muestra Configuración de Audio MIDI).
  - Se devuelven cuando la aplicación deja de usar el dispositivo.
  - Dos dispositivos con exactamente el mismo nombre no se pueden hacer
    bit-perfect.

## Qué ocurre en un dispositivo bit-perfect {#what-happens-on-a-bit-perfect-device}

- **Acceso exclusivo.** Nada más en el ordenador puede sonar en el
  dispositivo mientras la aplicación lo usa. Si se deniega el acceso
  exclusivo, el dispositivo sigue sonando, compartido, y el distintivo BP
  sigue apagado.
- **La frecuencia sigue al archivo.** El dispositivo se vuelve a abrir a la
  frecuencia de muestreo de una pista cuando esta empieza:
  - **Desde el silencio:** reproduces una pista, reanudas una cargada en
    pausa, preescuchas o disparas un cartucho mientras no suena nada en el
    dispositivo. Las pistas que solo están esperando (la siguiente de cada
    player) se vuelven a preparar a la nueva frecuencia.
  - **Entre dos pistas de un player:** la pista que suena se detiene (al
    final, o al momento con **Play / Siguiente** y **Anterior**), el
    dispositivo se vuelve a abrir y empieza la siguiente pista. Es el
    silencio breve de [Lo que cuesta](#what-it-costs).
  - El dispositivo solo se vuelve a abrir cuando ha reproducido todo lo que
    tenía en el búfer, así que el final de la pista anterior nunca se corta.
    Luego la reapertura tarda lo que el dispositivo necesite para arrancar.
    En conjunto suelen ser unas decenas de milisegundos; el inicio se
    retrasa eso mismo. La espera del búfer nunca es mayor de medio segundo
    (`tuning.play_out_max_ms` en el archivo de configuración, de 0 a
    2000 ms), diga lo que diga el dispositivo.
- **Cuando la frecuencia no puede seguir.** El dispositivo mantiene su
  frecuencia, y una pista con otra frecuencia se convierte entera (no es
  bit-perfect), cuando:
  - algo más suena en el dispositivo cuando empieza la pista: otro player (o
    uno en pausa con una pista), un cartucho o una preescucha;
  - el dispositivo rechazó antes esa frecuencia; mantiene la anterior;
  - la pista aún no se ha analizado, así que no se conoce su frecuencia.

  El cambio entre las dos pistas sigue siendo sin fundido encadenado.
- **Sin procesamiento, cuando nada lo pide.** Las muestras pasan sin cambios
  mientras no haya ningún fundido en curso (Stop con fundido, las rampas de
  pausa y reanudación, la parada breve sin chasquido de una pista) y no suene
  nada más en las mismas salidas (otro player, un cartucho, un tono de
  prueba). El volumen no importa: una salida bit-perfect lo mantiene al
  100 %.

## DSD {#dsd}

Un archivo DSD normalmente suena convertido a PCM, como cualquier otro
archivo. En cambio, un dispositivo bit-perfect puede recibir el flujo DSD
sin cambios.

**Los tres modos.** En la vista Avanzado de Configuración → Salidas de
audio, cada dispositivo que usa una salida tiene una opción **DSD** debajo
de su interruptor bit-perfect:
- **Convertir a PCM** (por defecto): el DSD se convierte, como en cualquier
  otro dispositivo.
- **DoP** (DSD sobre PCM): los bits DSD viajan dentro de muestras PCM de 24
  bits, que reconocen la mayoría de convertidores con DSD. Funciona en todos
  los sistemas.
- **DSD nativo** (solo Linux): DSD en bruto, para dispositivos ALSA `hw:`
  cuyo controlador declara un formato de muestra DSD.

Solo se ofrecen los modos que el dispositivo admite, y una línea debajo de
la opción explica por qué no los demás: el dispositivo no está conectado,
bit perfect está desactivado, el dispositivo no se puede abrir en
exclusiva, el DSD nativo necesita Linux, o el dispositivo no acepta DSD
nativo. Un modo guardado para un dispositivo que ahora no lo admite aparece
como PCM, que es lo que suena; el modo guardado vuelve cuando el
dispositivo puede volver a admitirlo. Cambiar un modo, el ajuste de mezcla o
el silencio DSD se aplica cuando no suena nada en el dispositivo, como los
demás ajustes de salida.

**Cuándo sale el DSD sin cambios.** Todo esto se debe cumplir cuando empieza
la pista:
- el dispositivo es bit-perfect, con acceso exclusivo, y su modo es DoP o
  DSD nativo;
- la pista es DSD (DSF o DFF), mono o estéreo, y se ha analizado (así se
  conoce su frecuencia DSD);
- no suena nada más en el dispositivo (otro player, un cartucho, un tono de
  prueba);
- la pista no tiene ganancia de [normalización de la
  sonoridad](loudness.md#bit-perfect-outputs): una cargada (por ejemplo en
  pausa) con ganancia antes de que el dispositivo pasara a bit-perfect la
  conserva, así que suena convertida, sin **DSD** ni **BP**;
- el dispositivo acepta el flujo. DoP necesita una frecuencia del
  dispositivo igual a la frecuencia DSD dividida entre 16 (176,4 kHz para
  DSD64, 352,8 kHz para DSD128, 705,6 kHz para DSD256) y un formato de 24 o
  32 bits. El DSD nativo necesita un dispositivo que acepte el formato DSD a
  esa frecuencia.

Cuando termina el DSD nativo, el dispositivo vuelve a PCM a la frecuencia
que tenía antes de la pista DSD, porque muchos convertidores aceptan DSD
nativo a frecuencias que no pueden reproducir como PCM (ningún convertidor
reproduce PCM a la frecuencia a la que funciona DSD512). Una pista que
continúa como PCM mantiene la frecuencia del flujo DSD si el dispositivo la
acepta como PCM, y si no continúa a la frecuencia anterior desde donde
estaba, como todo lo demás que suena en ese dispositivo; un fundido en curso
ahí termina al instante. El dispositivo nunca se queda en una frecuencia que
rechaza: si no se abre ninguna frecuencia (por ejemplo, porque el
dispositivo se desconectó en ese momento), la salida se pierde hasta que el
reintento automático la vuelve a abrir a la frecuencia anterior.

Si no se cumple, la pista se convierte a PCM y el registro explica por qué
(por ejemplo «something else plays on the device» o «the device refused
705600 Hz»). La preescucha y los cartuchos siempre se convierten.

Mientras el DSD sale sin cambios:
- el distintivo de la cabecera muestra **DSD** en lugar de **BP**;
- los vúmetros muestran el nivel de la conversión a PCM de la misma pista,
  así que funcionan como siempre;
- Stop y Stop con fundido detienen la pista al momento, sin fundido, porque
  un flujo DSD no admite fundidos;
- la pausa y la reanudación también actúan al momento, sin rampa.

**Silencio en los extremos.** Cada inicio, final y paso a PCM envía antes un
silencio DSD (200 ms por defecto), para que el convertidor se enganche sin
chasquido. La excepción es una pista DSD que continúa un flujo del mismo
tipo y la misma frecuencia DSD cuyo silencio aún está en curso: el
convertidor sigue enganchado, así que empieza sin silencio adicional. Por
eso una pista empieza ese tiempo más tarde, y un paso a PCM deja un hueco de
esa duración (un dispositivo con DSD nativo también reproduce lo que tiene
en el búfer antes de volver a abrirse como PCM, unos milisegundos más). Es **Silencio DSD** en Configuración → Salidas de audio,
Avanzado (de 0 a 2000 ms).

**Cuando otra fuente necesita el dispositivo.** **Cuando otra fuente
necesita una salida DSD**, en Configuración → Salidas de audio, elige qué
ocurre cuando otro player, un cartucho o un tono de prueba empieza en el
mismo dispositivo:
- **Seguir la pista DSD en PCM** (por defecto). El flujo pasa a PCM tras el
  silencio DSD, y la pista sigue, convertida, desde donde estaba. Lo mismo le
  pasa a la pista que sigue por sí sola (ver más abajo).
- **Mantener el DSD y silenciar las demás fuentes.** Nada interrumpe el flujo
  DSD. Las demás fuentes dirigidas al dispositivo quedan silenciadas hasta
  que acaba la pista DSD, y mientras tanto el player muestra el distintivo
  **Otras silenciadas**. La siguiente pista del propio player
  empieza cuando acaba la pista DSD: una pista PCM tras el silencio DSD, una
  pista DSD del mismo tipo y la misma frecuencia DSD al momento, continuando
  el flujo.

**Con el ajuste por defecto, un álbum no se mantiene en DSD.** Con *Seguir
la pista DSD en PCM*, solo sale como DSD una pista DSD que empieza en un
dispositivo inactivo. Las pistas que el player inicia por sí solo después
(al final de una pista) empiezan desde una
precarga, que siempre es PCM, así que el dispositivo pasa a PCM y suenan
convertidas. Una pista que inicias tú (Play, doble clic) vuelve a salir como
DSD si el dispositivo está inactivo o si el flujo DSD anterior sigue en su
silencio a la misma frecuencia DSD. Para mantener en DSD un álbum DSD
entero, elige *Mantener el DSD y silenciar las demás fuentes*. Así cada
pista del player sale como DSD, y la siguiente empieza cuando acaba la
anterior.

Si el dispositivo se pierde mientras suena DSD y vuelve sin poder llevarlo
(por ejemplo, sin acceso exclusivo), la pista sigue como PCM.

## El distintivo BP {#the-bp-badge}

El distintivo **BP** de la cabecera del player solo existe en un player
cuyo dispositivo Main funciona en bit-perfect. Se enciende (borde y texto
verdes; gris tenue cuando está apagado) mientras la pista actual llega sin
cambios a su dispositivo Main. Todo esto se debe cumplir:

- el dispositivo es bit-perfect y está abierto con acceso exclusivo;
- el dispositivo funciona a la frecuencia de muestreo de la pista;
- la pista es PCM entero sin pérdida (WAV, AIFF, FLAC, ALAC, WavPack o
  Monkey's Audio), mono o estéreo, y de 24 bits como máximo, y el formato
  del dispositivo admite su tamaño de muestra (un archivo de 24 bits en un
  dispositivo de 16 bits no es bit-perfect). El DSD se convierte, así que
  nunca enciende BP; cuando sale sin cambios (consulta [DSD](#dsd)), el
  distintivo muestra **DSD** en su lugar;
- la pista se ha analizado, porque así se conocen su frecuencia y su tamaño
  de muestra. Las pistas que analizó una versión anterior obtienen su
  formato al analizarse de nuevo (el aviso al arrancar, o Configuración →
  Análisis), o en cuanto un player las muestra o un cartucho las contiene;
- la pista suena a ganancia unitaria: una salida bit-perfect nunca se
  normaliza, pero una pista que empezó con una ganancia de normalización
  antes de que su dispositivo pasara a bit-perfect conserva esa ganancia
  hasta que se detiene, y BP se queda apagado para ella;
- el volumen se mantiene al 100 % en una salida bit-perfect, no hay ningún
  fundido en curso (Stop con fundido, las rampas de pausa y reanudación y la
  parada breve sin chasquido de una pista son los únicos fundidos que quedan
  en un player así) y no suena nada más en las mismas salidas.

Algunos archivos nunca se muestran como bit-perfect:
- **Archivos con pérdida** (MP3, AAC, Ogg Vorbis, Opus): sus muestras
  decodificadas no son los valores enteros que acepta un dispositivo.
- **Archivos de más de 24 bits:** el mezclador trabaja en coma flotante de
  32 bits, que lleva 24 bits exactos.
- **Archivos con más de dos canales:** se mezclan a estéreo.

## Comprobarlo tú mismo {#checking-it-yourself}

Para verificar una cadena de principio a fin:

1. Conecta la salida digital del dispositivo (S/PDIF, AES o loopback USB) a
   un grabador que capture bit a bit.
2. Reproduce un archivo de prueba sin pérdida sin que suene nada más.
3. Grábalo.
4. Compara la grabación con el archivo. Por ejemplo, con SoX, invierte uno y
   mézclalos: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav` después de
   alinear sus inicios. Todas las muestras de la diferencia deben ser cero.
5. Comprueba un cambio de frecuencia: en modo Continuo, con **Mezcla
   automática en el punto MIX** activada, reproduce una pista de 44,1 kHz que
   tenga un punto MIX seguida de una de 96 kHz. La primera suena hasta su
   cue-out sin fundido; tras un silencio breve el convertidor muestra
   96 kHz en la segunda. El distintivo BP se enciende en las dos.
6. Luego reproduce dos pistas a la misma frecuencia: la segunda sigue a la
   primera sin hueco y sin solape.

Las pruebas automáticas del proyecto comprueban esa misma propiedad dentro
de la aplicación, en un dispositivo simulado.

### DSD en un convertidor real {#dsd-on-a-real-converter}

Las pruebas automáticas solo comprueban el DSD en dispositivos simulados. El
proyecto no ha probado DoP ni DSD nativo en un convertidor real. Para
comprobar uno:
1. Pon el dispositivo en **DoP** (o en **DSD nativo** en Linux) y
   reproduce un archivo DSD sin que suene nada más. La cabecera debe
   mostrar **DSD**, y la pantalla del propio convertidor debería mostrar la
   frecuencia DSD (por ejemplo DSD64) en lugar de una frecuencia PCM. Un
   convertidor que muestra una frecuencia PCM o reproduce ruido no reconoce
   el flujo: vuelve a **Convertir a PCM**.
2. Escucha si hay un chasquido o una ráfaga de ruido al principio, en Stop,
   al final de la pista. Un chasquido significa que el
   convertidor necesita un **Silencio DSD** más largo (Configuración →
   Salidas de audio, Avanzado).
3. Inicia un cartucho u otro player en el mismo dispositivo, una vez con
   cada ajuste de mezcla, y comprueba el comportamiento descrito arriba.
4. En Linux, para comprobar el DSD nativo sin la aplicación, ejecuta
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test it -- --ignored conformance::native_dsd_on_a_real_device`.
   Abre el dispositivo en DSD nativo a DSD64 y reproduce un segundo de
   silencio DSD. Debe pasar, y el convertidor debería engancharse a DSD64.
5. Con archivos DSD en `test-music/`,
   `cargo test --release -p fp-engine --test it dsd_real_music:: -- --ignored`
   los reproduce a través del motor en un dispositivo simulado y compara las
   palabras con los bytes del archivo (consulta
   [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md),
   en inglés).
