# Superficies de control MIDI

Fauste Player pódese manexar desde controladores MIDI: controladores de pads
e faders, superficies de estilo DJ ou teclados. Os botóns de transporte e o
fader de volume de cada reprodutor pódense asignar a un botón, unha tecla ou
un fader, e os botóns con luces mostran o que fai cada reprodutor.

## Activalo {#turning-it-on}

![Configuración, MIDI: o interruptor para activar o MIDI, e a lista de accións cun botón Aprender cada unha](../../images/guide/settings-midi.png)

Abre **Configuración → MIDI** e marca **Usar superficies de control MIDI**. A
lista baixo **Portos de entrada** mostra todas as entradas MIDI que ten o
computador e se están conectadas. Só se abren os controladores que asignaches
(algúns sistemas dan un porto a un só programa á vez), máis todas as entradas
mentres aprendes un control; o programa nunca escoita os seus propios portos.
Os controladores pódense conectar e desconectar mentres o programa funciona:
cada un par de segundos búscanse os portos de novo, e un controlador que
volve conéctase polo seu nome, coas luces axustadas de novo.

En Linux, o MIDI pasa por ALSA: o teu usuario ten que poder abrir o
secuenciador (`/dev/snd/seq`, normalmente por pertencer ao grupo `audio`).

## Asignar un control {#binding-a-control}

Para cada reprodutor hai unha fila por acción: **Play / Seguinte**,
**Pausa**, **Stop**, **Stop con fundido**, **Reiniciar**, **Anterior**,
**CUE** e **Volume**.

1. Fai clic en **Aprender** na fila.
2. Preme o botón ou move o fader que queiras (mentres tanto di **Move un
   control…**). Un botón acepta unha tecla, un pad ou un botón que envía un
   cambio de control; **Volume** acepta un fader ou botón xiratorio (un cambio
   de control) ou un fader de pitch bend.
3. A fila mostra o dispositivo e o control, por exemplo
   `APC mini · Nota 36, canle 1`.

Se ese control xa estaba asignado a outra acción, pasa a esta. `Esc` ou
facer clic no botón de novo cancela a aprendizaxe. **Borrar** quita unha
asignación.

## Como se comportan os controis {#how-the-controls-behave}

- Un botón actúa cando se preme (unha tecla ou pad que baixa, ou un cambio de
  control que sobe polo medio do seu rango), exactamente coma o botón do
  reprodutor na pantalla. Un botón atenuado na pantalla non fai nada.
- Un fader move o volume na mesma escala que o fader da pantalla. Para evitar
  saltos, só toma o control cando chega ao volume actual ou o pasa (soft
  takeover); se o volume se cambia na pantalla, o fader ten que alcanzalo de
  novo. Non cambia nada ata que moves un control: iniciar o programa nunca
  envía nada a antena.
- Con **Acender os botóns (LED)** activado, os botóns asignados acéndense:
  Play mentres o reprodutor está en antena, Pausa pestanexando mentres está en
  pausa, CUE mentres se preescoita, e Stop, Stop con fundido, Reiniciar e
  Anterior mentres poden actuar. As luces van ao porto de saída do
  controlador co mesmo nome; pódese definir outro en `config.json`
  (`midi.devices`).
