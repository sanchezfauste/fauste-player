# Normalización de la sonoridad

Las pistas vienen de muchas fuentes y están masterizadas a niveles muy
distintos: un single pop reciente puede sonar el doble de fuerte que una
grabación de los años setenta o una cuña hablada. La **normalización de la
sonoridad** reproduce todas las pistas y todos los botones con la misma
sonoridad percibida, para que el operador no tenga que mover el fader de una
pista a la siguiente.

Está **activada en las instalaciones nuevas**; una configuración existente
conserva su ajuste actual (toda configuración guardada ya lo almacena).
Cámbiala en [Configuración → Sonoridad](settings.md#loudness).

![Configuración, Sonoridad: normalización activada, con el objetivo, el techo de true peak, la ganancia máxima y el ajuste manual máximo en sus valores por defecto](../../images/guide/settings-loudness.png)

## Cómo funciona {#how-it-works}

**Una ganancia fija por pista.** Cada pista recibe una única ganancia, en dB,
que se decide al cargar la pista y se mantiene hasta que se detiene. No se
comprime ni se limita nada: toda la pista se sube o se baja en la misma
cantidad, de modo que conserva la dinámica con la que se masterizó. El fader
del player actúa por encima, como antes.

**La medición.** El análisis en segundo plano mide cada archivo una vez:

- su **sonoridad integrada**, en LUFS: cuánto suena de fuerte el archivo
  entero, tal como la definen ITU-R BS.1770 y EBU R128 (los pasajes
  silenciosos y el silencio se descartan, así que un fundido final largo o
  una pausa no la bajan);
- su **true peak**, en dBTP: el nivel más alto que alcanza la señal entre
  muestras una vez convertida a analógico.

Los archivos nuevos se miden con su primer análisis. Los archivos añadidos
antes de que se midiera la sonoridad los mide una **pasada en segundo
plano** que se ejecuta cuando el análisis normal no tiene nada que hacer, sin
tocar sus marcadores. Se ejecuta con la normalización activada o
desactivada, así que las mediciones están listas cuando la actives. **Volver
a analizar** (en el menú de la playlist) mide un archivo otra vez, leyéndolo
aunque su tamaño y su fecha no hayan cambiado. Un archivo de menos de 0,4 s
no se mide nunca, y un archivo silencioso no tiene sonoridad: ambos suenan
sin cambios.

**La ganancia.** La ganancia lleva la sonoridad medida al **Objetivo**, más el
**Ajuste manual** propio de la pista (véase [Por pista](#per-track-exclude-and-trim)):

> ganancia = objetivo − sonoridad medida + ajuste

Una pista más fuerte que el objetivo se baja lo que haga falta. Una pista
más baja se sube, pero nunca más de la **Ganancia máxima**, y nunca tanto que
su true peak supere el **Techo de true peak**: no hay limitador, así que el
techo es lo que evita que una pista subida sature. Una pista cuyo true peak
ya está por encima del techo no se sube en absoluto.

Ejemplos, con los valores por defecto (objetivo −16 LUFS, techo −1 dBTP,
ganancia máxima 12 dB):

| Pista | Medida | True peak | Ganancia |
|---|---|---|---|
| Un máster moderno fuerte | −9,0 LUFS | +0,5 dBTP | −7,0 dB |
| Un máster antiguo más bajo | −20,0 LUFS | −10,0 dBTP | +4,0 dB |
| Una pista baja con picos altos | −22,0 LUFS | −3,0 dBTP | +2,0 dB (limitada por el techo) |
| Una grabación muy baja | −35,0 LUFS | −20,0 dBTP | +12,0 dB (limitada por la ganancia máxima) |

**Cuándo se decide la ganancia.** Cuando se carga una pista: preparada como
siguiente de un player, iniciada, cargada en pausa, preescuchada en CUE o
disparada como botón. Una pista que suena o está en pausa **conserva su
ganancia** hasta que se detiene, a través de saltos y pausas. Cambiar los
ajustes, la medición de una pista o su ajuste manual se aplica la próxima vez
que se cargue la pista; la siguiente pista que ya espera en un player se
prepara de nuevo con la ganancia nueva, así que empieza con ella.

**Preescucha.** La preescucha en CUE y la de botones reproducen una pista con
la misma ganancia que tiene en antena, así que lo que oyes en los
auriculares coincide con el programa.

**Vúmetros.** Los vúmetros y su lectura de sonoridad miden después de la
ganancia, así que muestran el nivel normalizado. El **Objetivo de
sonoridad** propio de los vúmetros (Configuración → Vúmetros) solo colorea la
lectura; no cambia ninguna ganancia.

## Salidas bit-perfect {#bit-perfect-outputs}

Un player o un botón cuya salida es [bit-perfect](bit-perfect.md) **nunca se
normaliza**: siempre suena a ganancia unitaria (0 dB), sean cuales sean los
ajustes, porque cualquier ganancia cambiaría las muestras que recibe el
dispositivo. Con la normalización activada, su cabecera muestra un
distintivo ámbar **0.0 dB** cuyo tooltip dice «Salida bit-perfect: las
pistas suenan sin alterar». Lo mismo vale para la preescucha en un
dispositivo Cue bit-perfect, mientras que la salida Main del player, si no
es bit-perfect, sigue recibiendo la ganancia.

Una pista que ya está sonando (o en pausa) cuando su dispositivo pasa a
bit-perfect conserva la ganancia con la que empezó, y el distintivo **BP**
sigue apagado para ella. La siguiente pista suena a 0 dB y bit-perfect. Una
pista DSD en esa situación suena convertida a PCM con su ganancia, nunca
como un flujo DSD sin cambios.

## La ganancia en la cabecera del player {#the-gain-in-the-player-header}

Mientras un player reproduce o está en pausa, un distintivo a la izquierda de
**BP** muestra la ganancia de la pista en antena. El distintivo tiene un
borde de su color y siempre el mismo ancho. Pasa el cursor por encima para
ver el motivo.

| Distintivo | Significado |
|---|---|
| **+4.0 dB**, **−7.0 dB** (azul) | La pista está normalizada con esta ganancia. El tooltip es el desglose de la ganancia que se describe más abajo. Si los ajustes, la medición o el ajuste manual cambiaron después de cargar la pista, solo dice que la pista conserva la ganancia fijada al cargarla, y muestra esa ganancia. |
| **0.0 dB** (ámbar) | La pista suena sin cambios. El tooltip es el mismo desglose, y su línea **Ganancia automática** explica por qué: aún no tiene medición (o es demasiado corta para medirla), la salida es bit-perfect o la pista está excluida de la normalización. |
| Sin distintivo | La normalización estaba desactivada cuando empezó la pista, la pista es silenciosa o el player está detenido. Un player bit-perfect tampoco muestra ninguno mientras la normalización está desactivada. |

El distintivo muestra la ganancia que pidió el player cuando empezó la pista.

### El desglose de la ganancia {#the-gain-breakdown}

Siempre que el programa explica la ganancia de una pista muestra el mismo
bloque de líneas, con la etiqueta a la izquierda y el valor a la derecha: el
tooltip del distintivo de ganancia, la ventana emergente de sonoridad (bajo
sus controles), el editor de botones y el tooltip de pista de la tabla de la
playlist.

| Línea | Significado |
|---|---|
| **Sonoridad integrada** | La sonoridad medida de la pista, en LUFS (o «sin medir»). |
| **True peak** | Su true peak más alto, en dBTP (solo cuando está medido). |
| **Objetivo** | La sonoridad a la que se lleva cada pista (Configuración → Sonoridad). |
| **Ganancia automática** | Objetivo menos sonoridad medida, antes del ajuste y de cualquier límite, o «ninguna» y el motivo: excluida, salida bit-perfect, normalización desactivada, aún sin medir o silenciosa. |
| **Ajuste manual** | El ajuste manual de la pista en uso, cuando no es 0 (con el valor guardado y el máximo cuando un **Ajuste manual máximo** reducido lo limita). |
| **Ganancia deseada** y **Limitada por** | Solo cuando el techo de true peak o la ganancia máxima han frenado la ganancia: la ganancia automática más el ajuste (se muestra cuando hay ajuste) y el límite que la detuvo. |
| **Ganancia resultante** | Siempre la última, bajo una línea y en negrita: la ganancia con la que suena la pista. 0.0 dB cuando no está normalizada. |

Con la normalización desactivada, una nota sobre la ganancia resultante dice
que no cambia nada en antena y que el ajuste manual se aplica cuando se
active.

## Por pista: excluir y ajustar {#per-track-exclude-and-trim}

Algunas pistas no deberían seguir el objetivo: una identificación de
emisora masterizada fuerte a propósito, un tono de prueba, una pista cuya
medición no concuerda con cómo suena (la voz suele sonar más fuerte que la
música con los mismos LUFS). Cada pista tiene dos ajustes para eso:

- **Excluir de la normalización:** la pista suena siempre sin cambios.
- **Ajuste manual:** dB que se suman a la ganancia de la pista, desde − hasta
  + el **Ajuste manual máximo** (Configuración → Sonoridad), en pasos de
  0,5 dB. Haz doble clic en el deslizador para devolverlo a 0. El ajuste se
  atenúa mientras la pista está excluida.

El deslizador es un **ajuste**: un desplazamiento sobre la ganancia
automática, así que marca 0 dB en todas las pistas que no has editado. Por
eso la ventana emergente y el editor de botones muestran el [desglose de la
ganancia](#the-gain-breakdown) junto con sus controles: la línea **Ganancia
resultante** sigue al deslizador mientras lo mueves. En la ventana
emergente los controles van primero y el desglose debajo, así que el
deslizador se queda donde está mientras el desglose crece o se encoge.
Mientras la pista suena, la ventana emergente añade que un cambio se aplica
desde su próxima carga.

Dónde cambiarlos:

- **En una forma de onda:** haz clic derecho en la forma de onda de un player
  (o en la de su ventana CUE) y elige **Sonoridad…**. La ventana emergente
  nombra la pista y muestra su medición. Sigue editando esa pista aunque el
  player pase a otra; ciérrala con su botón **×**. Se cierra sola si se
  quita la pista.
- **En una fila de la playlist:** haz clic derecho en la fila y elige
  **Sonoridad…**: la misma ventana emergente, para la pista de esa fila.
- **En el editor de botones:** Configuración → Cartuchera, haz clic en un
  botón: los mismos dos controles están bajo **Parar los demás al
  dispararlo**, con la medición bajo la etiqueta.
- **Por control remoto:** `PUT /api/v1/tracks/<id>/loudness` (véase
  [Control remoto](remote-control.md)).

Los ajustes pertenecen a la pista y se guardan con ella. Un cambio se aplica
la próxima vez que se carga la pista, como cualquier otro cambio.

**El tooltip de pista** de la tabla de la playlist muestra las etiquetas de la
pista y, bajo una línea, el mismo [desglose de la ganancia](#the-gain-breakdown),
calculado para el player al que pertenece esa lista (un player bit-perfect la
reproduce sin alterar).

## Elegir los ajustes {#choosing-the-settings}

- **Objetivo.** −16 LUFS va bien para la mayoría de los programas musicales
  en antena y en línea. −23 LUFS (EBU R128) o −24 LUFS (ATSC A/85) dejan más
  margen, pero entonces las pistas bajas necesitan subidas mayores, que el
  techo y la ganancia máxima pueden frenar. Elige un objetivo cercano a lo
  fuerte que ya suena tu material.
- **Techo de true peak.** −1 dBTP deja margen para la conversión y para los
  codificadores con pérdida que haya más adelante. Bájalo cuando un
  codificador de streaming sature.
- **Ganancia máxima.** Evita que una grabación muy baja (o casi silenciosa) se
  suba hasta el ruido. 0 dB solo baja las pistas fuertes.
- **Ajuste manual máximo.** Hasta dónde puede llegar el ajuste por pista.
  Reducirlo limita los ajustes ya hechos sin cambiarlos, hasta que vuelvas a
  editar la sonoridad de esa pista: entonces la edición guarda el ajuste
  limitado.

## Los estándares detrás de los valores por defecto {#standards-behind-the-defaults}

- **Medición.** Sonoridad integrada y true peak tal como los define
  ITU-R BS.1770 (ponderación K, con puerta).
- **Objetivo −16 LUFS.** Sigue las recomendaciones de la AES para streaming y
  distribución en línea. AES TD1008 (2021) sustituye a la anterior AES
  TD1004.1.15-10, que recomendaba de −16 a −20 LUFS para streaming. No es un
  valor de radiodifusión: EBU R128 usa −23 LUFS y ATSC A/85 usa −24 LKFS.
  Usa uno de ellos cuando una especificación de entrega lo exija (EBU R128
  para el intercambio de programas en la radiodifusión europea, ATSC A/85
  para la televisión digital de EE. UU.), y −16 para un stream o un programa
  en línea.
- **Techo de true peak −1 dBTP.** El true peak máximo de EBU R128, usado
  también por las recomendaciones de la AES. Los codificadores con pérdida
  pueden sobrepasar al decodificar, así que un stream enviado por un
  codificador de baja tasa de bits puede querer un techo más bajo, como
  −2 dBTP (una práctica habitual, no parte de esos estándares).
- **Ganancia máxima y ajuste manual máximo, 12 dB.** Decisiones del producto
  sin ningún estándar detrás: suficiente para subir una grabación baja y para
  corregir una pista de oído, sin tanto como para subir el ruido o para que
  el ajuste anule el objetivo.
- **La normalización está activada en las instalaciones nuevas.** Actualizar
  la aplicación conserva el ajuste que ya tenía tu configuración, así que una
  instalación que la tenía desactivada sigue así hasta que la actives.
- **El objetivo de sonoridad de los vúmetros** (Configuración → Vúmetros) es
  un ajuste aparte, por defecto el valor de R128, −23 LUFS. La normalización
  no lo usa.
