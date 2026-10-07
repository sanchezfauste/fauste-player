# Superficies de control MIDI

Fauste Player se puede manejar desde controladores MIDI: controladores de
pads y faders, superficies de estilo DJ o teclados. Los botones de
transporte y el fader de volumen de cada player se pueden asignar a un
botón, una tecla o un fader, y los botones con luces muestran lo que hace
cada player.

## Activarlo {#turning-it-on}

![Configuración, MIDI: el interruptor para activar MIDI y la lista de acciones, cada una con un botón Aprender](../../images/guide/settings-midi.png)

Abre **Configuración → MIDI** y marca **Usar superficies de control MIDI**.
La lista de **Puertos de entrada** muestra todas las entradas MIDI del
ordenador y si están conectadas. Solo se abren los controladores que has
asignado (algunos sistemas dan un puerto a un solo programa a la vez), más
todas las entradas mientras aprendes un control; el programa nunca escucha
sus propios puertos. Los controladores se pueden conectar o desconectar con
el programa en marcha: cada par de segundos se vuelven a buscar los
puertos, y un controlador que vuelve se conecta por su nombre, con sus luces
otra vez encendidas como corresponde.

En Linux, MIDI pasa por ALSA: tu usuario debe poder abrir el secuenciador
(`/dev/snd/seq`, normalmente por estar en el grupo `audio`).

## Asignar un control {#binding-a-control}

Para cada player hay una fila por acción: **Play / Siguiente**, **Pausa**,
**Stop**, **Stop con fundido**, **Reiniciar**, **Anterior**, **CUE** y
**Volumen**.

1. Haz clic en **Aprender** en la fila.
2. Pulsa el botón o mueve el fader que quieras (mientras tanto dice **Mueve
   un control…**). Un botón acepta una tecla, un pad o un botón que envía un
   control change; **Volumen** acepta un fader o un potenciómetro (un control
   change) o un fader de pitch bend.
3. La fila muestra el dispositivo y el control, por ejemplo
   `APC mini · Nota 36, canal 1`.

Si ese control ya estaba asignado a otra acción, pasa a esta. `Esc` o volver
a hacer clic en el botón cancela el aprendizaje. **Borrar** elimina una
asignación.

## Cómo se comportan los controles {#how-the-controls-behave}

- Un botón actúa al pulsarlo (una tecla o un pad que baja, o un control
  change que sube y pasa por la mitad de su rango), exactamente igual que el
  botón del player en pantalla. Un botón que está atenuado en pantalla no
  hace nada.
- Un fader mueve el volumen en la misma escala que el fader de la pantalla.
  Para evitar saltos, solo toma el control cuando alcanza o supera el
  volumen actual (soft takeover); si el volumen se cambia en pantalla, el
  fader tiene que volver a alcanzarlo. No cambia nada hasta que mueves un
  control: arrancar el programa nunca pone nada al aire.
- Con **Encender los botones (LED)** activado, los botones asignados se
  iluminan: Play mientras el player suena, Pausa parpadeando mientras está
  en pausa, CUE mientras preescucha, y Stop, Stop con fundido, Reiniciar y
  Anterior mientras pueden actuar. Las luces van al puerto de salida del
  controlador que tiene el mismo nombre; se puede fijar otro en
  `config.json` (`midi.devices`).
